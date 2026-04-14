#[tokio::main]
async fn main() -> anyhow::Result<()> {
    test_igd().await
}

#[derive(Debug)]
struct Entry {
    remote_host: String,
    external_port: u16,
    protocol: &'static str,
    internal_port: u16,
    internal_client: String,
    enabled: bool,
    port_mapping_description: String,
    lease_duration: u32,
}

impl Entry {
    fn from(entry: igd::PortMappingEntry) -> Self {
        Entry {
            remote_host: entry.remote_host,
            external_port: entry.external_port,
            protocol: match entry.protocol {
                igd::PortMappingProtocol::TCP => "tcp",
                igd::PortMappingProtocol::UDP => "udp"
            },
            internal_port: entry.internal_port,
            internal_client: entry.internal_client,
            enabled: entry.enabled,
            port_mapping_description: entry.port_mapping_description,
            lease_duration: entry.lease_duration,
        }
    }
}

async fn test_igd() -> anyhow::Result<()> {
    let gateway = igd::aio::search_gateway(Default::default()).await?;
    // println!("gateway control schema: {:#?}", gateway.control_schema);
    // let external_address = gateway.get_external_ip().await?;
    let mut i = 0;
    loop {
        let entry = gateway.get_generic_port_mapping_entry(i).await?;
        println!("entry #{i}: {:#?}", Entry::from(entry));
        i+=1;
    }
    // gateway.remove_port(igd::PortMappingProtocol::TCP, 65479).await?;
    // Ok(())
}
