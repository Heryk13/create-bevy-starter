#[expect(
    dead_code,
    reason = "nothing reads the choices until the prompts ask for them (spec step 5)"
)]
mod choices;

fn main() {
    println!("Hello, world!");
}
