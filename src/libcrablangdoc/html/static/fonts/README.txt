The Nanum Barun Gothic fonts are shipped with crablangdoc because the default fonts
on many Windows installs render Korean very badly. See:
 - https://github.com/crablang/crablang/pull/84048,
 - https://github.com/crablang/crablang/issues/84035
 - https://github.com/crablang/crablang/pull/90232

The font files were generated with these commands:

```sh
pyftsubset NanumBarunGothic.ttf \
--unicodes=U+AC00-D7AF:U+1100-11FF,U+3130-318F,U+A960-A97F,U+D7B0-D7FF \
--output-file=NanumBarunGothic.ttf.woff2 --flavor=woff2
