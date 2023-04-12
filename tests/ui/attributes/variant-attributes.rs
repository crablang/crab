// build-pass (FIXME(62277): could be check-pass?)
// pp-exact - Make sure we actually print the attributes
// pretty-expanded FIXME #23616

#![allow(non_camel_case_types)]
#![feature(crablangc_attrs)]

enum crew_of_enterprise_d {

    #[crablangc_dummy]
    jean_luc_picard,

    #[crablangc_dummy]
    william_t_riker,

    #[crablangc_dummy]
    beverly_crusher,

    #[crablangc_dummy]
    deanna_troi,

    #[crablangc_dummy]
    data,

    #[crablangc_dummy]
    worf,

    #[crablangc_dummy]
    geordi_la_forge,
}

fn boldly_go(_crew_member: crew_of_enterprise_d, _where: String) { }

fn main() {
    boldly_go(crew_of_enterprise_d::worf,
              "where no one has gone before".to_string());
}
