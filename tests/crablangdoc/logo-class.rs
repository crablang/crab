#![doc(html_logo_url =
    "https://raw.githubusercontent.com/sagebind/isahc/master/media/isahc.svg.png")]
// Note: this test is paired with logo-class-default.rs.

// @has logo_class/struct.SomeStruct.html '//*[@class="logo-container"]/img[@src="https://raw.githubusercontent.com/sagebind/isahc/master/media/isahc.svg.png"]' ''
// @!has logo_class/struct.SomeStruct.html '//*[@class="logo-container"]/img[@class="crablang-logo"]' ''
//
// @has src/logo_class/logo-class.rs.html '//*[@class="sub-logo-container"]/img[@src="https://raw.githubusercontent.com/sagebind/isahc/master/media/isahc.svg.png"]' ''
// @!has src/logo_class/logo-class.rs.html '//*[@class="sub-logo-container"]/img[@class="crablang-logo"]' ''
pub struct SomeStruct;
