[![Build Status](https://travis-ci.org/crablang/crablang-installer.svg?branch=master)](https://travis-ci.org/crablang/crablang-installer)

A generator for the install.sh script commonly used to install CrabLang in
Unix environments. It is used By CrabLang, Cargo, and is intended to be
used by a future combined installer of CrabLang + Cargo.

# Usage

```
./gen-installer.sh --product-name=CrabLang \
                   --rel-manifest-dir=crablanglib \
                   --success-message=CrabLang-is-ready-to-roll. \
                   --image-dir=./install-image \
                   --work-dir=./temp \
                   --output-dir=./dist \
                   --non-installed-overlay=./overlay \
                   --package-name=crablangc-nightly-i686-apple-darwin \
                   --component-name=crablangc \
                   --legacy-manifest-dirs=crablanglib \
                   --bulk-dirs=share/doc
```

Or, to just generate the script.

```
./gen-install-script.sh --product-name=CrabLang \
                        --rel-manifest-dir=crablanglib \
                        --success-message=CrabLang-is-ready-to-roll. \
                        --output-script=install.sh \
                        --legacy-manifest-dirs=crablanglib
```

*Note: the dashes in `success-message` are converted to spaces. The
script's argument handling is broken with spaces.*

To combine installers.

```
./combine-installers.sh --product-name=CrabLang \
                        --rel-manifest-dir=crablanglib \
                        --success-message=CrabLang-is-ready-to-roll. \
                        --work-dir=./temp \
                        --output-dir=./dist \
                        --non-installed-overlay=./overlay \
                        --package-name=crablangc-nightly-i686-apple-darwin \
                        --legacy-manifest-dirs=crablanglib \
                        --input-tarballs=./crablangc.tar.gz,cargo.tar.gz
```

# Future work

* Make install.sh not have to be customized, pull it's data from a
  config file.
* Be more resiliant to installation failures, particularly if the disk
  is full.
* Pre-install and post-uninstall scripts.
* Allow components to depend on or contradict other components.
* Sanity check that expected destination dirs (bin, lib, share exist)?
* Add --docdir flag. Is there a standard name for this?
* Remove empty directories on uninstall.
* Detect mismatches in --prefix, --mandir, etc. in follow-on
  installs/uninstalls.
* Fix argument handling for spaces.
* Add --bindir.

# License

This software is distributed under the terms of both the MIT license
and/or the Apache License (Version 2.0), at your option.

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.
