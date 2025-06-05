use spiel::client::ProviderProxy;
use std::error::Error;
use zbus::names::OwnedBusName;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let session = zbus::Connection::session().await?;
    let s = zbus::fdo::DBusProxy::new(&session).await?;
    let mut active = OwnedBusName::try_from("org.espeak.Speech.Provider")?;
    for i in s.list_activatable_names().await? {
        if i.contains("Speech.Provider") {
            println!("SP: {i:?}");
            active = i;
        }
    }
    println!("AC: {active}");
    for i in s.interfaces().await? {
        println!("INT: {i:?}");
    }
    let prov = ProviderProxy::new(
        &session,
        active.clone(),
        "/".to_owned() + &active.as_str().replace(".", "/"),
    )
    .await?;
    println!("Name: {:?}", prov.name().await?);
    for i in prov.voices().await? {
        println!("{i:?}");
    }
    Ok(())
}
