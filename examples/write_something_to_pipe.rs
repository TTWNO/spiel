use spiel::client::ProviderProxy;
use std::error::Error;
use std::os::fd::AsFd;
use std::time::Duration;
use tokio::io::ErrorKind;
use tokio::net::UnixStream;
use tokio::time::timeout;
use zbus::names::OwnedBusName;
use zbus::zvariant::Fd;

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
    let (read, write) = UnixStream::pair()?;
    prov.synthesize(
        Fd::Borrowed(write.as_fd()),
        "This is a test using Spiel! Wahahaa!",
        "m6",
        1.0,
        1.0,
        false,
        "en-US",
    )
    .await?;
    loop {
        // Wait for the socket to be readable
        let _ = timeout(Duration::from_secs(1), read.readable()).await?;

        let mut buf = Vec::with_capacity(4096);

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match read.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {n} bytes");
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    Ok(())
}
