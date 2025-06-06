use std::error::Error;

use spiel::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let client = Client::new().await?;
	let providers = client.list_providers().await?;
	for provider in providers {
		let pname = provider.name().await?;
		print!("{pname} provides: ");
		let mut names: Vec<_> =
			provider.voices().await?.into_iter().map(|voice| voice.name).collect();
		names.sort();
		println!("{names:?}");
	}
	Ok(())
}
