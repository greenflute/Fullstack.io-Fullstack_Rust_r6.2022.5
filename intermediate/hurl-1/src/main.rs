use structopt::StructOpt;

mod app;

fn main() -> Result<(), ()> {
    let mut app = app::App::from_args();
    app.validate()?;
    println!("{:#?}", app);
    Ok(())
}
