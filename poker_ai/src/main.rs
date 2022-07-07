
use s

fn main() {
    println!("Starting test");
    let players: Vec<Box<dyn ActionSupplier>> = vec![
        Box::new(AlwaysCall{}),
        Box::new(AlwaysMinRaise{}),
        Box::new(AlwaysFold{}),
        Box::new(AlwaysCall{}),
        Box::new(AlwaysMinRaise{}),
        Box::new(AlwaysFold{}),
    ];
    let res = full_test(&players, 6, 1000000);
    println!("Finished test, results:\n{}", res);
}

struct AlwaysCall {}
impl ActionSupplier for AlwaysCall {
    fn get_action(&self, _rs: &RoundState) -> f32 {
        return 0.0;
    }
    fn name(&self) -> &str {
        "AlwaysCall"
    }
}

struct AlwaysMinRaise {}
impl ActionSupplier for AlwaysMinRaise {
    fn get_action(&self, _rs: &RoundState) -> f32 {
        return 1e-20;
    }
    fn name(&self) -> &str {
        "AlwaysMinRaise"
    }
}

struct AlwaysFold {}
impl ActionSupplier for AlwaysFold {
    fn get_action(&self, _rs: &RoundState) -> f32 {
        return -1.0;
    }
    fn name(&self) -> &str {
        "AlwaysFold"
    }
}
