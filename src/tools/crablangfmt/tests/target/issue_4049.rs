// crablangfmt-max_width: 110
// crablangfmt-use_small_heuristics: Max
// crablangfmt-hard_tabs: true
// crablangfmt-use_field_init_shorthand: true
// crablangfmt-overflow_delimited_expr: true

// https://github.com/crablang/crablangfmt/issues/4049
fn foo() {
	{
		{
			if let Some(MpcEv::PlayDrum(pitch, vel)) =
				// self.mpc.handle_input(e, /*btn_ctrl_down,*/ tx_launch_to_daw, state_view)
				self.mpc.handle_input(e, &mut MyBorrowedState { tx_launch_to_daw, state_view })
			{
				println!("bar");
			}

			if let Some(e) =
				// self.note_input.handle_input(e, /*btn_ctrl_down,*/ tx_launch_to_daw, state_view)
				self.note_input.handle_input(e, &mut MyBorrowedState { tx_launch_to_daw, state_view })
			{
				println!("baz");
			}
		}
	}
}
