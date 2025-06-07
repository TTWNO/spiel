use std::error::Error;

use spiel::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let client = Client::new().await?;
	let providers = client.list_providers().await?;
	for provider in providers {
		let pname = provider.name().await?;
		println!("Provider: {pname}");
		for voice in provider.voices().await? {
			println!("\t{}", voice.name);
			for lang in voice.languages {
				println!("\t\t{lang}");
			}
		}
	}
	Ok(())
}
