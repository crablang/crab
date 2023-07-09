use super::*;
use std::fmt::Write;
use std::{borrow::Borrow, cmp, iter, ops::Bound};

#[cfg(feature = "randomize")]
use rand::{seq::SliceRandom, SeedableRng};
#[cfg(feature = "randomize")]
use rand_xoshiro::Xoshiro128StarStar;

use tracing::debug;

pub trait LayoutCalculator {
    type TargetDataLayoutRef: Borrow<TargetDataLayout>;

    fn delay_bug(&self, txt: String);
    fn current_data_layout(&self) -> Self::TargetDataLayoutRef;

    fn scalar_pair(&self, a: Scalar, b: Scalar) -> LayoutS {
        let dl = self.current_data_layout();
        let dl = dl.borrow();
        let b_align = b.align(dl);
        let align = a.align(dl).max(b_align).max(dl.aggregate_align);
        let b_offset = a.size(dl).align_to(b_align.abi);
        let size = (b_offset + b.size(dl)).align_to(align.abi);

        // HACK(nox): We iter on `b` and then `a` because `max_by_key`
        // returns the last maximum.
        let largest_niche = Niche::from_scalar(dl, b_offset, b)
            .into_iter()
            .chain(Niche::from_scalar(dl, Size::ZERO, a))
            .max_by_key(|niche| niche.available(dl));

        LayoutS {
            variants: Variants::Single { index: FIRST_VARIANT },
            fields: FieldsShape::Arbitrary {
                offsets: [Size::ZERO, b_offset].into(),
                memory_index: [0, 1].into(),
            },
            abi: Abi::ScalarPair(a, b),
            largest_niche,
            align,
            size,
        }
    }

    fn univariant(
        &self,
        dl: &TargetDataLayout,
        fields: &IndexSlice<FieldIdx, Layout<'_>>,
        repr: &ReprOptions,
        kind: StructKind,
    ) -> Option<LayoutS> {
        let layout = univariant(self, dl, fields, repr, kind, NicheBias::Start);
        // Enums prefer niches close to the beginning or the end of the variants so that other (smaller)
        // data-carrying variants can be packed into the space after/before the niche.
        // If the default field ordering does not give us a niche at the front then we do a second
        // run and bias niches to the right and then check which one is closer to one of the struct's
        // edges.
        if let Some(layout) = &layout {
            // Don't try to calculate an end-biased layout for unsizable structs,
            // otherwise we could end up with different layouts for
            // Foo<Type> and Foo<dyn Trait> which would break unsizing
            if !matches!(kind, StructKind::MaybeUnsized) {
                if let Some(niche) = layout.largest_niche {
                    let head_space = niche.offset.bytes();
                    let niche_length = niche.value.size(dl).bytes();
                    let tail_space = layout.size.bytes() - head_space - niche_length;

                    // This may end up doing redundant work if the niche is already in the last field
                    // (e.g. a trailing bool) and there is tail padding. But it's non-trivial to get
                    // the unpadded size so we try anyway.
                    if fields.len() > 1 && head_space != 0 && tail_space > 0 {
                        let alt_layout = univariant(self, dl, fields, repr, kind, NicheBias::End)
                            .expect("alt layout should always work");
                        let niche = alt_layout
                            .largest_niche
                            .expect("alt layout should have a niche like the regular one");
                        let alt_head_space = niche.offset.bytes();
                        let alt_niche_len = niche.value.size(dl).bytes();
                        let alt_tail_space =
                            alt_layout.size.bytes() - alt_head_space - alt_niche_len;

                        debug_assert_eq!(layout.size.bytes(), alt_layout.size.bytes());

                        let prefer_alt_layout =
                            alt_head_space > head_space && alt_head_space > tail_space;

                        debug!(
                            "sz: {}, default_niche_at: {}+{}, default_tail_space: {}, alt_niche_at/head_space: {}+{}, alt_tail: {}, num_fields: {}, better: {}\n\
                            layout: {}\n\
                            alt_layout: {}\n",
                            layout.size.bytes(),
                            head_space,
                            niche_length,
                            tail_space,
                            alt_head_space,
                            alt_niche_len,
                            alt_tail_space,
                            layout.fields.count(),
                            prefer_alt_layout,
                            format_field_niches(&layout, &fields, &dl),
                            format_field_niches(&alt_layout, &fields, &dl),
                        );

                        if prefer_alt_layout {
                            return Some(alt_layout);
                        }
                    }
                }
            }
        }
        layout
    }

    fn layout_of_never_type(&self) -> LayoutS {
        let dl = self.current_data_layout();
        let dl = dl.borrow();
        LayoutS {
            variants: Variants::Single { index: FIRST_VARIANT },
            fields: FieldsShape::Primitive,
            abi: Abi::Uninhabited,
            largest_niche: None,
            align: dl.i8_align,
            size: Size::ZERO,
        }
    }

    fn layout_of_struct_or_enum(
        &self,
        repr: &ReprOptions,
        variants: &IndexSlice<VariantIdx, IndexVec<FieldIdx, Layout<'_>>>,
        is_enum: bool,
        is_unsafe_cell: bool,
        scalar_valid_range: (Bound<u128>, Bound<u128>),
        discr_range_of_repr: impl Fn(i128, i128) -> (Integer, bool),
        discriminants: impl Iterator<Item = (VariantIdx, i128)>,
        dont_niche_optimize_enum: bool,
        always_sized: bool,
    ) -> Option<LayoutS> {
        let dl = self.current_data_layout();
        let dl = dl.borrow();

        let scalar_unit = |value: Primitive| {
            let size = value.size(dl);
            assert!(size.bits() <= 128);
            Scalar::Initialized { value, valid_range: WrappingRange::full(size) }
        };

        // A variant is absent if it's uninhabited and only has ZST fields.
        // Present uninhabited variants only require space for their fields,
        // but *not* an encoding of the discriminant (e.g., a tag value).
        // See issue #49298 for more details on the need to leave space
        // for non-ZST uninhabited data (mostly partial initialization).
        let absent = |fields: &IndexSlice<FieldIdx, Layout<'_>>| {
            let uninhabited = fields.iter().any(|f| f.abi().is_uninhabited());
            let is_zst = fields.iter().all(|f| f.0.is_zst());
            uninhabited && is_zst
        };
        let (present_first, present_second) = {
            let mut present_variants = variants
                .iter_enumerated()
                .filter_map(|(i, v)| if absent(v) { None } else { Some(i) });
            (present_variants.next(), present_variants.next())
        };
        let present_first = match present_first {
            Some(present_first) => present_first,
            // Uninhabited because it has no variants, or only absent ones.
            None if is_enum => {
                return Some(self.layout_of_never_type());
            }
            // If it's a struct, still compute a layout so that we can still compute the
            // field offsets.
            None => FIRST_VARIANT,
        };

        let is_struct = !is_enum ||
                    // Only one variant is present.
                    (present_second.is_none() &&
                        // Representation optimizations are allowed.
                        !repr.inhibit_enum_layout_opt());
        if is_struct {
            // Struct, or univariant enum equivalent to a struct.
            // (Typechecking will reject discriminant-sizing attrs.)

            let v = present_first;
            let kind = if is_enum || variants[v].is_empty() || always_sized {
                StructKind::AlwaysSized
            } else {
                StructKind::MaybeUnsized
            };

            let mut st = self.univariant(dl, &variants[v], repr, kind)?;
            st.variants = Variants::Single { index: v };

            if is_unsafe_cell {
                let hide_niches = |scalar: &mut _| match scalar {
                    Scalar::Initialized { value, valid_range } => {
                        *valid_range = WrappingRange::full(value.size(dl))
                    }
                    // Already doesn't have any niches
                    Scalar::Union { .. } => {}
                };
                match &mut st.abi {
                    Abi::Uninhabited => {}
                    Abi::Scalar(scalar) => hide_niches(scalar),
                    Abi::ScalarPair(a, b) => {
                        hide_niches(a);
                        hide_niches(b);
                    }
                    Abi::Vector { element, count: _ } => hide_niches(element),
                    Abi::Aggregate { sized: _ } => {}
                }
                st.largest_niche = None;
                return Some(st);
            }

            let (start, end) = scalar_valid_range;
            match st.abi {
                Abi::Scalar(ref mut scalar) | Abi::ScalarPair(ref mut scalar, _) => {
                    // Enlarging validity ranges would result in missed
                    // optimizations, *not* wrongly assuming the inner
                    // value is valid. e.g. unions already enlarge validity ranges,
                    // because the values may be uninitialized.
                    //
                    // Because of that we only check that the start and end
                    // of the range is representable with this scalar type.

                    let max_value = scalar.size(dl).unsigned_int_max();
                    if let Bound::Included(start) = start {
                        // FIXME(eddyb) this might be incorrect - it doesn't
                        // account for wrap-around (end < start) ranges.
                        assert!(start <= max_value, "{start} > {max_value}");
                        scalar.valid_range_mut().start = start;
                    }
                    if let Bound::Included(end) = end {
                        // FIXME(eddyb) this might be incorrect - it doesn't
                        // account for wrap-around (end < start) ranges.
                        assert!(end <= max_value, "{end} > {max_value}");
                        scalar.valid_range_mut().end = end;
                    }

                    // Update `largest_niche` if we have introduced a larger niche.
                    let niche = Niche::from_scalar(dl, Size::ZERO, *scalar);
                    if let Some(niche) = niche {
                        match st.largest_niche {
                            Some(largest_niche) => {
                                // Replace the existing niche even if they're equal,
                                // because this one is at a lower offset.
                                if largest_niche.available(dl) <= niche.available(dl) {
                                    st.largest_niche = Some(niche);
                                }
                            }
                            None => st.largest_niche = Some(niche),
                        }
                    }
                }
                _ => assert!(
                    start == Bound::Unbounded && end == Bound::Unbounded,
                    "nonscalar layout for layout_scalar_valid_range type: {:#?}",
                    st,
                ),
            }

            return Some(st);
        }

        // At this point, we have handled all unions and
        // structs. (We have also handled univariant enums
        // that allow representation optimization.)
        assert!(is_enum);

        // Until we've decided whether to use the tagged or
        // niche filling LayoutS, we don't want to intern the
        // variant layouts, so we can't store them in the
        // overall LayoutS. Store the overall LayoutS
        // and the variant LayoutSs here until then.
        struct TmpLayout {
            layout: LayoutS,
            variants: IndexVec<VariantIdx, LayoutS>,
        }

        let calculate_niche_filling_layout = || -> Option<TmpLayout> {
            if dont_niche_optimize_enum {
                return None;
            }

            if variants.len() < 2 {
                return None;
            }

            let mut align = dl.aggregate_align;
            let mut variant_layouts = variants
                .iter_enumerated()
                .map(|(j, v)| {
                    let mut st = self.univariant(dl, v, repr, StructKind::AlwaysSized)?;
                    st.variants = Variants::Single { index: j };

                    align = align.max(st.align);

                    Some(st)
                })
                .collect::<Option<IndexVec<VariantIdx, _>>>()?;

            let largest_variant_index = variant_layouts
                .iter_enumerated()
                .max_by_key(|(_i, layout)| layout.size.bytes())
                .map(|(i, _layout)| i)?;

            let all_indices = variants.indices();
            let needs_disc =
                |index: VariantIdx| index != largest_variant_index && !absent(&variants[index]);
            let niche_variants = all_indices.clone().find(|v| needs_disc(*v)).unwrap()
                ..=all_indices.rev().find(|v| needs_disc(*v)).unwrap();

            let count = niche_variants.size_hint().1.unwrap() as u128;

            // Find the field with the largest niche
            let (field_index, niche, (niche_start, niche_scalar)) = variants[largest_variant_index]
                .iter()
                .enumerate()
                .filter_map(|(j, field)| Some((j, field.largest_niche()?)))
                .max_by_key(|(_, niche)| niche.available(dl))
                .and_then(|(j, niche)| Some((j, niche, niche.reserve(dl, count)?)))?;
            let niche_offset =
                niche.offset + variant_layouts[largest_variant_index].fields.offset(field_index);
            let niche_size = niche.value.size(dl);
            let size = variant_layouts[largest_variant_index].size.align_to(align.abi);

            let all_variants_fit = variant_layouts.iter_enumerated_mut().all(|(i, layout)| {
                if i == largest_variant_index {
                    return true;
                }

                layout.largest_niche = None;

                if layout.size <= niche_offset {
                    // This variant will fit before the niche.
                    return true;
                }

                // Determine if it'll fit after the niche.
                let this_align = layout.align.abi;
                let this_offset = (niche_offset + niche_size).align_to(this_align);

                if this_offset + layout.size > size {
                    return false;
                }

                // It'll fit, but we need to make some adjustments.
                match layout.fields {
                    FieldsShape::Arbitrary { ref mut offsets, .. } => {
                        for (j, offset) in offsets.iter_enumerated_mut() {
                            if !variants[i][j].0.is_zst() {
                                *offset += this_offset;
                            }
                        }
                    }
                    _ => {
                        panic!("Layout of fields should be Arbitrary for variants")
                    }
                }

                // It can't be a Scalar or ScalarPair because the offset isn't 0.
                if !layout.abi.is_uninhabited() {
                    layout.abi = Abi::Aggregate { sized: true };
                }
                layout.size += this_offset;

                true
            });

            if !all_variants_fit {
                return None;
            }

            let largest_niche = Niche::from_scalar(dl, niche_offset, niche_scalar);

            let others_zst = variant_layouts
                .iter_enumerated()
                .all(|(i, layout)| i == largest_variant_index || layout.size == Size::ZERO);
            let same_size = size == variant_layouts[largest_variant_index].size;
            let same_align = align == variant_layouts[largest_variant_index].align;

            let abi = if variant_layouts.iter().all(|v| v.abi.is_uninhabited()) {
                Abi::Uninhabited
            } else if same_size && same_align && others_zst {
                match variant_layouts[largest_variant_index].abi {
                    // When the total alignment and size match, we can use the
                    // same ABI as the scalar variant with the reserved niche.
                    Abi::Scalar(_) => Abi::Scalar(niche_scalar),
                    Abi::ScalarPair(first, second) => {
                        // Only the niche is guaranteed to be initialised,
                        // so use union layouts for the other primitive.
                        if niche_offset == Size::ZERO {
                            Abi::ScalarPair(niche_scalar, second.to_union())
                        } else {
                            Abi::ScalarPair(first.to_union(), niche_scalar)
                        }
                    }
                    _ => Abi::Aggregate { sized: true },
                }
            } else {
                Abi::Aggregate { sized: true }
            };

            let layout = LayoutS {
                variants: Variants::Multiple {
                    tag: niche_scalar,
                    tag_encoding: TagEncoding::Niche {
                        untagged_variant: largest_variant_index,
                        niche_variants,
                        niche_start,
                    },
                    tag_field: 0,
                    variants: IndexVec::new(),
                },
                fields: FieldsShape::Arbitrary {
                    offsets: [niche_offset].into(),
                    memory_index: [0].into(),
                },
                abi,
                largest_niche,
                size,
                align,
            };

            Some(TmpLayout { layout, variants: variant_layouts })
        };

        let niche_filling_layout = calculate_niche_filling_layout();

        let (mut min, mut max) = (i128::MAX, i128::MIN);
        let discr_type = repr.discr_type();
        let bits = Integer::from_attr(dl, discr_type).size().bits();
        for (i, mut val) in discriminants {
            if variants[i].iter().any(|f| f.abi().is_uninhabited()) {
                continue;
            }
            if discr_type.is_signed() {
                // sign extend the raw representation to be an i128
                val = (val << (128 - bits)) >> (128 - bits);
            }
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }
        // We might have no inhabited variants, so pretend there's at least one.
        if (min, max) == (i128::MAX, i128::MIN) {
            min = 0;
            max = 0;
        }
        assert!(min <= max, "discriminant range is {}...{}", min, max);
        let (min_ity, signed) = discr_range_of_repr(min, max); //Integer::repr_discr(tcx, ty, &repr, min, max);

        let mut align = dl.aggregate_align;
        let mut size = Size::ZERO;

        // We're interested in the smallest alignment, so start large.
        let mut start_align = Align::from_bytes(256).unwrap();
        assert_eq!(Integer::for_align(dl, start_align), None);

        // repr(C) on an enum tells us to make a (tag, union) layout,
        // so we need to grow the prefix alignment to be at least
        // the alignment of the union. (This value is used both for
        // determining the alignment of the overall enum, and the
        // determining the alignment of the payload after the tag.)
        let mut prefix_align = min_ity.align(dl).abi;
        if repr.c() {
            for fields in variants {
                for field in fields {
                    prefix_align = prefix_align.max(field.align().abi);
                }
            }
        }

        // Create the set of structs that represent each variant.
        let mut layout_variants = variants
            .iter_enumerated()
            .map(|(i, field_layouts)| {
                let mut st = self.univariant(
                    dl,
                    field_layouts,
                    repr,
                    StructKind::Prefixed(min_ity.size(), prefix_align),
                )?;
                st.variants = Variants::Single { index: i };
                // Find the first field we can't move later
                // to make room for a larger discriminant.
                for field_idx in st.fields.index_by_increasing_offset() {
                    let field = &field_layouts[FieldIdx::from_usize(field_idx)];
                    if !field.0.is_zst() || field.align().abi.bytes() != 1 {
                        start_align = start_align.min(field.align().abi);
                        break;
                    }
                }
                size = cmp::max(size, st.size);
                align = align.max(st.align);
                Some(st)
            })
            .collect::<Option<IndexVec<VariantIdx, _>>>()?;

        // Align the maximum variant size to the largest alignment.
        size = size.align_to(align.abi);

        if size.bytes() >= dl.obj_size_bound() {
            return None;
        }

        let typeck_ity = Integer::from_attr(dl, repr.discr_type());
        if typeck_ity < min_ity {
            // It is a bug if Layout decided on a greater discriminant size than typeck for
            // some reason at this point (based on values discriminant can take on). Mostly
            // because this discriminant will be loaded, and then stored into variable of
            // type calculated by typeck. Consider such case (a bug): typeck decided on
            // byte-sized discriminant, but layout thinks we need a 16-bit to store all
            // discriminant values. That would be a bug, because then, in codegen, in order
            // to store this 16-bit discriminant into 8-bit sized temporary some of the
            // space necessary to represent would have to be discarded (or layout is wrong
            // on thinking it needs 16 bits)
            panic!(
                "layout decided on a larger discriminant type ({:?}) than typeck ({:?})",
                min_ity, typeck_ity
            );
            // However, it is fine to make discr type however large (as an optimisation)
            // after this point – we’ll just truncate the value we load in codegen.
        }

        // Check to see if we should use a different type for the
        // discriminant. We can safely use a type with the same size
        // as the alignment of the first field of each variant.
        // We increase the size of the discriminant to avoid LLVM copying
        // padding when it doesn't need to. This normally causes unaligned
        // load/stores and excessive memcpy/memset operations. By using a
        // bigger integer size, LLVM can be sure about its contents and
        // won't be so conservative.

        // Use the initial field alignment
        let mut ity = if repr.c() || repr.int.is_some() {
            min_ity
        } else {
            Integer::for_align(dl, start_align).unwrap_or(min_ity)
        };

        // If the alignment is not larger than the chosen discriminant size,
        // don't use the alignment as the final size.
        if ity <= min_ity {
            ity = min_ity;
        } else {
            // Patch up the variants' first few fields.
            let old_ity_size = min_ity.size();
            let new_ity_size = ity.size();
            for variant in &mut layout_variants {
                match variant.fields {
                    FieldsShape::Arbitrary { ref mut offsets, .. } => {
                        for i in offsets {
                            if *i <= old_ity_size {
                                assert_eq!(*i, old_ity_size);
                                *i = new_ity_size;
                            }
                        }
                        // We might be making the struct larger.
                        if variant.size <= old_ity_size {
                            variant.size = new_ity_size;
                        }
                    }
                    _ => panic!(),
                }
            }
        }

        let tag_mask = ity.size().unsigned_int_max();
        let tag = Scalar::Initialized {
            value: Int(ity, signed),
            valid_range: WrappingRange {
                start: (min as u128 & tag_mask),
                end: (max as u128 & tag_mask),
            },
        };
        let mut abi = Abi::Aggregate { sized: true };

        if layout_variants.iter().all(|v| v.abi.is_uninhabited()) {
            abi = Abi::Uninhabited;
        } else if tag.size(dl) == size {
            // Make sure we only use scalar layout when the enum is entirely its
            // own tag (i.e. it has no padding nor any non-ZST variant fields).
            abi = Abi::Scalar(tag);
        } else {
            // Try to use a ScalarPair for all tagged enums.
            let mut common_prim = None;
            let mut common_prim_initialized_in_all_variants = true;
            for (field_layouts, layout_variant) in iter::zip(variants, &layout_variants) {
                let FieldsShape::Arbitrary { ref offsets, .. } = layout_variant.fields else {
                    panic!();
                };
                let mut fields = iter::zip(field_layouts, offsets).filter(|p| !p.0.0.is_zst());
                let (field, offset) = match (fields.next(), fields.next()) {
                    (None, None) => {
                        common_prim_initialized_in_all_variants = false;
                        continue;
                    }
                    (Some(pair), None) => pair,
                    _ => {
                        common_prim = None;
                        break;
                    }
                };
                let prim = match field.abi() {
                    Abi::Scalar(scalar) => {
                        common_prim_initialized_in_all_variants &=
                            matches!(scalar, Scalar::Initialized { .. });
                        scalar.primitive()
                    }
                    _ => {
                        common_prim = None;
                        break;
                    }
                };
                if let Some(pair) = common_prim {
                    // This is pretty conservative. We could go fancier
                    // by conflating things like i32 and u32, or even
                    // realising that (u8, u8) could just cohabit with
                    // u16 or even u32.
                    if pair != (prim, offset) {
                        common_prim = None;
                        break;
                    }
                } else {
                    common_prim = Some((prim, offset));
                }
            }
            if let Some((prim, offset)) = common_prim {
                let prim_scalar = if common_prim_initialized_in_all_variants {
                    scalar_unit(prim)
                } else {
                    // Common prim might be uninit.
                    Scalar::Union { value: prim }
                };
                let pair = self.scalar_pair(tag, prim_scalar);
                let pair_offsets = match pair.fields {
                    FieldsShape::Arbitrary { ref offsets, ref memory_index } => {
                        assert_eq!(memory_index.raw, [0, 1]);
                        offsets
                    }
                    _ => panic!(),
                };
                if pair_offsets[FieldIdx::from_u32(0)] == Size::ZERO
                    && pair_offsets[FieldIdx::from_u32(1)] == *offset
                    && align == pair.align
                    && size == pair.size
                {
                    // We can use `ScalarPair` only when it matches our
                    // already computed layout (including `#[repr(C)]`).
                    abi = pair.abi;
                }
            }
        }

        // If we pick a "clever" (by-value) ABI, we might have to adjust the ABI of the
        // variants to ensure they are consistent. This is because a downcast is
        // semantically a NOP, and thus should not affect layout.
        if matches!(abi, Abi::Scalar(..) | Abi::ScalarPair(..)) {
            for variant in &mut layout_variants {
                // We only do this for variants with fields; the others are not accessed anyway.
                // Also do not overwrite any already existing "clever" ABIs.
                if variant.fields.count() > 0 && matches!(variant.abi, Abi::Aggregate { .. }) {
                    variant.abi = abi;
                    // Also need to bump up the size and alignment, so that the entire value fits in here.
                    variant.size = cmp::max(variant.size, size);
                    variant.align.abi = cmp::max(variant.align.abi, align.abi);
                }
            }
        }

        let largest_niche = Niche::from_scalar(dl, Size::ZERO, tag);

        let tagged_layout = LayoutS {
            variants: Variants::Multiple {
                tag,
                tag_encoding: TagEncoding::Direct,
                tag_field: 0,
                variants: IndexVec::new(),
            },
            fields: FieldsShape::Arbitrary {
                offsets: [Size::ZERO].into(),
                memory_index: [0].into(),
            },
            largest_niche,
            abi,
            align,
            size,
        };

        let tagged_layout = TmpLayout { layout: tagged_layout, variants: layout_variants };

        let mut best_layout = match (tagged_layout, niche_filling_layout) {
            (tl, Some(nl)) => {
                // Pick the smaller layout; otherwise,
                // pick the layout with the larger niche; otherwise,
                // pick tagged as it has simpler codegen.
                use cmp::Ordering::*;
                let niche_size =
                    |tmp_l: &TmpLayout| tmp_l.layout.largest_niche.map_or(0, |n| n.available(dl));
                match (tl.layout.size.cmp(&nl.layout.size), niche_size(&tl).cmp(&niche_size(&nl))) {
                    (Greater, _) => nl,
                    (Equal, Less) => nl,
                    _ => tl,
                }
            }
            (tl, None) => tl,
        };

        // Now we can intern the variant layouts and store them in the enum layout.
        best_layout.layout.variants = match best_layout.layout.variants {
            Variants::Multiple { tag, tag_encoding, tag_field, .. } => {
                Variants::Multiple { tag, tag_encoding, tag_field, variants: best_layout.variants }
            }
            _ => panic!(),
        };
        Some(best_layout.layout)
    }

    fn layout_of_union(
        &self,
        repr: &ReprOptions,
        variants: &IndexSlice<VariantIdx, IndexVec<FieldIdx, Layout<'_>>>,
    ) -> Option<LayoutS> {
        let dl = self.current_data_layout();
        let dl = dl.borrow();
        let mut align = if repr.pack.is_some() { dl.i8_align } else { dl.aggregate_align };

        if let Some(repr_align) = repr.align {
            align = align.max(AbiAndPrefAlign::new(repr_align));
        }

        // If all the non-ZST fields have the same ABI and union ABI optimizations aren't
        // disabled, we can use that common ABI for the union as a whole.
        struct AbiMismatch;
        let mut common_non_zst_abi_and_align = if repr.inhibit_union_abi_opt() {
            // Can't optimize
            Err(AbiMismatch)
        } else {
            Ok(None)
        };

        let mut size = Size::ZERO;
        let only_variant = &variants[FIRST_VARIANT];
        for field in only_variant {
            assert!(field.0.is_sized());

            align = align.max(field.align());
            size = cmp::max(size, field.size());

            if field.0.is_zst() {
                // Nothing more to do for ZST fields
                continue;
            }

            if let Ok(common) = common_non_zst_abi_and_align {
                // Discard valid range information and allow undef
                let field_abi = field.abi().to_union();

                if let Some((common_abi, common_align)) = common {
                    if common_abi != field_abi {
                        // Different fields have different ABI: disable opt
                        common_non_zst_abi_and_align = Err(AbiMismatch);
                    } else {
                        // Fields with the same non-Aggregate ABI should also
                        // have the same alignment
                        if !matches!(common_abi, Abi::Aggregate { .. }) {
                            assert_eq!(
                                common_align,
                                field.align().abi,
                                "non-Aggregate field with matching ABI but differing alignment"
                            );
                        }
                    }
                } else {
                    // First non-ZST field: record its ABI and alignment
                    common_non_zst_abi_and_align = Ok(Some((field_abi, field.align().abi)));
                }
            }
        }

        if let Some(pack) = repr.pack {
            align = align.min(AbiAndPrefAlign::new(pack));
        }

        // If all non-ZST fields have the same ABI, we may forward that ABI
        // for the union as a whole, unless otherwise inhibited.
        let abi = match common_non_zst_abi_and_align {
            Err(AbiMismatch) | Ok(None) => Abi::Aggregate { sized: true },
            Ok(Some((abi, _))) => {
                if abi.inherent_align(dl).map(|a| a.abi) != Some(align.abi) {
                    // Mismatched alignment (e.g. union is #[repr(packed)]): disable opt
                    Abi::Aggregate { sized: true }
                } else {
                    abi
                }
            }
        };

        Some(LayoutS {
            variants: Variants::Single { index: FIRST_VARIANT },
            fields: FieldsShape::Union(NonZeroUsize::new(only_variant.len())?),
            abi,
            largest_niche: None,
            align,
            size: size.align_to(align.abi),
        })
    }
}

/// Determines towards which end of a struct layout optimizations will try to place the best niches.
enum NicheBias {
    Start,
    End,
}

fn univariant(
    this: &(impl LayoutCalculator + ?Sized),
    dl: &TargetDataLayout,
    fields: &IndexSlice<FieldIdx, Layout<'_>>,
    repr: &ReprOptions,
    kind: StructKind,
    niche_bias: NicheBias,
) -> Option<LayoutS> {
    let pack = repr.pack;
    let mut align = if pack.is_some() { dl.i8_align } else { dl.aggregate_align };
    let mut inverse_memory_index: IndexVec<u32, FieldIdx> = fields.indices().collect();
    let optimize = !repr.inhibit_struct_field_reordering_opt();
    if optimize && fields.len() > 1 {
        let end = if let StructKind::MaybeUnsized = kind { fields.len() - 1 } else { fields.len() };
        let optimizing = &mut inverse_memory_index.raw[..end];
        let fields_excluding_tail = &fields.raw[..end];

        // If `-Z randomize-layout` was enabled for the type definition we can shuffle
        // the field ordering to try and catch some code making assumptions about layouts
        // we don't guarantee
        if repr.can_randomize_type_layout() && cfg!(feature = "randomize") {
            #[cfg(feature = "randomize")]
            {
                // `ReprOptions.layout_seed` is a deterministic seed that we can use to
                // randomize field ordering with
                let mut rng = Xoshiro128StarStar::seed_from_u64(repr.field_shuffle_seed.as_u64());

                // Shuffle the ordering of the fields
                optimizing.shuffle(&mut rng);
            }
            // Otherwise we just leave things alone and actually optimize the type's fields
        } else {
            // To allow unsizing `&Foo<Type>` -> `&Foo<dyn Trait>`, the layout of the struct must
            // not depend on the layout of the tail.
            let max_field_align =
                fields_excluding_tail.iter().map(|f| f.align().abi.bytes()).max().unwrap_or(1);
            let largest_niche_size = fields_excluding_tail
                .iter()
                .filter_map(|f| f.largest_niche())
                .map(|n| n.available(dl))
                .max()
                .unwrap_or(0);

            // Calculates a sort key to group fields by their alignment or possibly some size-derived
            // pseudo-alignment.
            let alignment_group_key = |layout: Layout<'_>| {
                if let Some(pack) = pack {
                    // return the packed alignment in bytes
                    layout.align().abi.min(pack).bytes()
                } else {
                    // returns log2(effective-align).
                    // This is ok since `pack` applies to all fields equally.
                    // The calculation assumes that size is an integer multiple of align, except for ZSTs.
                    //
                    let align = layout.align().abi.bytes();
                    let size = layout.size().bytes();
                    let niche_size = layout.largest_niche().map(|n| n.available(dl)).unwrap_or(0);
                    // group [u8; 4] with align-4 or [u8; 6] with align-2 fields
                    let size_as_align = align.max(size).trailing_zeros();
                    let size_as_align = if largest_niche_size > 0 {
                        match niche_bias {
                            // Given `A(u8, [u8; 16])` and `B(bool, [u8; 16])` we want to bump the array
                            // to the front in the first case (for aligned loads) but keep the bool in front
                            // in the second case for its niches.
                            NicheBias::Start => max_field_align.trailing_zeros().min(size_as_align),
                            // When moving niches towards the end of the struct then for
                            // A((u8, u8, u8, bool), (u8, bool, u8)) we want to keep the first tuple
                            // in the align-1 group because its bool can be moved closer to the end.
                            NicheBias::End if niche_size == largest_niche_size => {
                                align.trailing_zeros()
                            }
                            NicheBias::End => size_as_align,
                        }
                    } else {
                        size_as_align
                    };
                    size_as_align as u64
                }
            };

            match kind {
                StructKind::AlwaysSized | StructKind::MaybeUnsized => {
                    // Currently `LayoutS` only exposes a single niche so sorting is usually sufficient
                    // to get one niche into the preferred position. If it ever supported multiple niches
                    // then a more advanced pick-and-pack approach could provide better results.
                    // But even for the single-niche cache it's not optimal. E.g. for
                    // A(u32, (bool, u8), u16) it would be possible to move the bool to the front
                    // but it would require packing the tuple together with the u16 to build a 4-byte
                    // group so that the u32 can be placed after it without padding. This kind
                    // of packing can't be achieved by sorting.
                    optimizing.sort_by_key(|&x| {
                        let f = fields[x];
                        let field_size = f.size().bytes();
                        let niche_size = f.largest_niche().map_or(0, |n| n.available(dl));
                        let niche_size_key = match niche_bias {
                            // large niche first
                            NicheBias::Start => !niche_size,
                            // large niche last
                            NicheBias::End => niche_size,
                        };
                        let inner_niche_offset_key = match niche_bias {
                            NicheBias::Start => f.largest_niche().map_or(0, |n| n.offset.bytes()),
                            NicheBias::End => f.largest_niche().map_or(0, |n| {
                                !(field_size - n.value.size(dl).bytes() - n.offset.bytes())
                            }),
                        };

                        (
                            // Place ZSTs first to avoid "interesting offsets", especially with only one
                            // or two non-ZST fields. This helps Scalar/ScalarPair layouts.
                            !f.0.is_zst(),
                            // Then place largest alignments first.
                            cmp::Reverse(alignment_group_key(f)),
                            // Then prioritize niche placement within alignment group according to
                            // `niche_bias_start`.
                            niche_size_key,
                            // Then among fields with equally-sized niches prefer the ones
                            // closer to the start/end of the field.
                            inner_niche_offset_key,
                        )
                    });
                }

                StructKind::Prefixed(..) => {
                    // Sort in ascending alignment so that the layout stays optimal
                    // regardless of the prefix.
                    // And put the largest niche in an alignment group at the end
                    // so it can be used as discriminant in jagged enums
                    optimizing.sort_by_key(|&x| {
                        let f = fields[x];
                        let niche_size = f.largest_niche().map_or(0, |n| n.available(dl));
                        (alignment_group_key(f), niche_size)
                    });
                }
            }

            // FIXME(Kixiron): We can always shuffle fields within a given alignment class
            //                 regardless of the status of `-Z randomize-layout`
        }
    }
    // inverse_memory_index holds field indices by increasing memory offset.
    // That is, if field 5 has offset 0, the first element of inverse_memory_index is 5.
    // We now write field offsets to the corresponding offset slot;
    // field 5 with offset 0 puts 0 in offsets[5].
    // At the bottom of this function, we invert `inverse_memory_index` to
    // produce `memory_index` (see `invert_mapping`).
    let mut sized = true;
    let mut offsets = IndexVec::from_elem(Size::ZERO, &fields);
    let mut offset = Size::ZERO;
    let mut largest_niche = None;
    let mut largest_niche_available = 0;
    if let StructKind::Prefixed(prefix_size, prefix_align) = kind {
        let prefix_align =
            if let Some(pack) = pack { prefix_align.min(pack) } else { prefix_align };
        align = align.max(AbiAndPrefAlign::new(prefix_align));
        offset = prefix_size.align_to(prefix_align);
    }
    for &i in &inverse_memory_index {
        let field = &fields[i];
        if !sized {
            this.delay_bug(format!(
                "univariant: field #{} comes after unsized field",
                offsets.len(),
            ));
        }

        if field.0.is_unsized() {
            sized = false;
        }

        // Invariant: offset < dl.obj_size_bound() <= 1<<61
        let field_align = if let Some(pack) = pack {
            field.align().min(AbiAndPrefAlign::new(pack))
        } else {
            field.align()
        };
        offset = offset.align_to(field_align.abi);
        align = align.max(field_align);

        debug!("univariant offset: {:?} field: {:#?}", offset, field);
        offsets[i] = offset;

        if let Some(mut niche) = field.largest_niche() {
            let available = niche.available(dl);
            // Pick up larger niches.
            let prefer_new_niche = match niche_bias {
                NicheBias::Start => available > largest_niche_available,
                // if there are several niches of the same size then pick the last one
                NicheBias::End => available >= largest_niche_available,
            };
            if prefer_new_niche {
                largest_niche_available = available;
                niche.offset += offset;
                largest_niche = Some(niche);
            }
        }

        offset = offset.checked_add(field.size(), dl)?;
    }
    if let Some(repr_align) = repr.align {
        align = align.max(AbiAndPrefAlign::new(repr_align));
    }
    debug!("univariant min_size: {:?}", offset);
    let min_size = offset;
    // As stated above, inverse_memory_index holds field indices by increasing offset.
    // This makes it an already-sorted view of the offsets vec.
    // To invert it, consider:
    // If field 5 has offset 0, offsets[0] is 5, and memory_index[5] should be 0.
    // Field 5 would be the first element, so memory_index is i:
    // Note: if we didn't optimize, it's already right.
    let memory_index = if optimize {
        inverse_memory_index.invert_bijective_mapping()
    } else {
        debug_assert!(inverse_memory_index.iter().copied().eq(fields.indices()));
        inverse_memory_index.into_iter().map(FieldIdx::as_u32).collect()
    };
    let size = min_size.align_to(align.abi);
    let mut abi = Abi::Aggregate { sized };
    // Unpack newtype ABIs and find scalar pairs.
    if sized && size.bytes() > 0 {
        // All other fields must be ZSTs.
        let mut non_zst_fields = fields.iter_enumerated().filter(|&(_, f)| !f.0.is_zst());

        match (non_zst_fields.next(), non_zst_fields.next(), non_zst_fields.next()) {
            // We have exactly one non-ZST field.
            (Some((i, field)), None, None) => {
                // Field fills the struct and it has a scalar or scalar pair ABI.
                if offsets[i].bytes() == 0 && align.abi == field.align().abi && size == field.size()
                {
                    match field.abi() {
                        // For plain scalars, or vectors of them, we can't unpack
                        // newtypes for `#[repr(C)]`, as that affects C ABIs.
                        Abi::Scalar(_) | Abi::Vector { .. } if optimize => {
                            abi = field.abi();
                        }
                        // But scalar pairs are Rust-specific and get
                        // treated as aggregates by C ABIs anyway.
                        Abi::ScalarPair(..) => {
                            abi = field.abi();
                        }
                        _ => {}
                    }
                }
            }

            // Two non-ZST fields, and they're both scalars.
            (Some((i, a)), Some((j, b)), None) => {
                match (a.abi(), b.abi()) {
                    (Abi::Scalar(a), Abi::Scalar(b)) => {
                        // Order by the memory placement, not source order.
                        let ((i, a), (j, b)) = if offsets[i] < offsets[j] {
                            ((i, a), (j, b))
                        } else {
                            ((j, b), (i, a))
                        };
                        let pair = this.scalar_pair(a, b);
                        let pair_offsets = match pair.fields {
                            FieldsShape::Arbitrary { ref offsets, ref memory_index } => {
                                assert_eq!(memory_index.raw, [0, 1]);
                                offsets
                            }
                            _ => panic!(),
                        };
                        if offsets[i] == pair_offsets[FieldIdx::from_usize(0)]
                            && offsets[j] == pair_offsets[FieldIdx::from_usize(1)]
                            && align == pair.align
                            && size == pair.size
                        {
                            // We can use `ScalarPair` only when it matches our
                            // already computed layout (including `#[repr(C)]`).
                            abi = pair.abi;
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }
    if fields.iter().any(|f| f.abi().is_uninhabited()) {
        abi = Abi::Uninhabited;
    }
    Some(LayoutS {
        variants: Variants::Single { index: FIRST_VARIANT },
        fields: FieldsShape::Arbitrary { offsets, memory_index },
        abi,
        largest_niche,
        align,
        size,
    })
}

fn format_field_niches(
    layout: &LayoutS,
    fields: &IndexSlice<FieldIdx, Layout<'_>>,
    dl: &TargetDataLayout,
) -> String {
    let mut s = String::new();
    for i in layout.fields.index_by_increasing_offset() {
        let offset = layout.fields.offset(i);
        let f = fields[i.into()];
        write!(s, "[o{}a{}s{}", offset.bytes(), f.align().abi.bytes(), f.size().bytes()).unwrap();
        if let Some(n) = f.largest_niche() {
            write!(
                s,
                " n{}b{}s{}",
                n.offset.bytes(),
                n.available(dl).ilog2(),
                n.value.size(dl).bytes()
            )
            .unwrap();
        }
        write!(s, "] ").unwrap();
    }
    s
}
