use crate::analyser::Analyser;

pub struct FBCScrubber {
    analyser: Analyser,
}
impl FBCScrubber {
    pub fn new() -> FBCScrubber {
        FBCScrubber {
            analyser: Analyser::default()
        }
    }
}