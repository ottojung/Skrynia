mod adversarial;
mod guest;
mod run;
mod wire;
pub fn run() -> Result<(), String> {
    run::run()?;
    adversarial::run()?;
    println!("v3_program0_kernel_smoke=PASS");
    Ok(())
}
