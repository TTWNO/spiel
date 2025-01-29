use zbus;
use spiel::proxy::ProviderProxy;
use zbus::names::WellKnownName;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let session = zbus::Connection::session().await?;
	let s = zbus::fdo::DBusProxy::new(&session).await?;
	let mut active = WellKnownName::try_from("org.espeak.Speech.Provider")?;
	for i in s.list_activatable_names().await? {
		if i.contains("Speech.Provider") {
			println!("{:?}", i);
		}
	}
	for i in s.interfaces().await? {
		println!("{:?}", i);
	}
	let x = s.start_service_by_name(active, 0).await?;
	let prov = ProviderProxy::new(&session, "org.espeak.Speech.Provider", "/org/espeak/Speech/Provider").await?;
	println!("Name: {:?}", prov.name().await?);
	//for i in prov.voices().await? {
	//	println!("{:?}", i);
	//}
	Ok(())
}
