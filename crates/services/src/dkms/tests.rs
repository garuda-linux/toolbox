use toolbox_system::ProbeError;

use super::{Dkms, DkmsFailure, dkms_answer};

#[test]
fn a_dkms_answer_becomes_its_state() {
    assert_eq!(
        dkms_answer(Err(ProbeError::Unavailable)),
        Ok(Dkms::NotInstalled)
    );
    assert_eq!(
        dkms_answer(Err(ProbeError::fake("no output"))),
        Err(DkmsFailure("no output: fake source".to_owned()))
    );
    assert_eq!(dkms_answer(Ok(Vec::new())), Ok(Dkms::Modules(Vec::new())));
}
