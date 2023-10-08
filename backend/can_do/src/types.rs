#[derive(Default, Clone)]
pub struct Grantee {
    // public fields as they're only used inside this crate
    pub grantee_of: Vec<usize>,
    pub grantees: Vec<usize>,
    pub actions: Vec<usize>,
    pub is_root: bool, // roots are not removed when compacting
}

#[derive(Default, Clone)]
pub struct Action {
    // public fields as they're only used inside this crate
    pub grantees: Vec<usize>,
    pub main_action_of: Vec<usize>,
    pub sub_action_of: Vec<usize>,
}
