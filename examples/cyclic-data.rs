use ethercat::{SlaveId, PdoInfo, PdoEntryIndex, PdoEntryInfo, Master, MasterAccess, SyncInfo, SlaveAddr, Offset, DomainIndex};
use ethercat_esi::EtherCatInfo;
use std::{collections::HashMap,io::{self, prelude::*}, time::Duration, thread, env, fs::File};

type BitLen = u8;

pub fn main() -> Result<(), io::Error> {
    let args: Vec<_> = env::args().collect();
    let file_name = match args.len() {
        2 => &args[1],
        _=> {
            println!("usage: {} ESI-FILE", env!("CARGO_PKG_NAME"));
            return Ok(())
        }
    };

    let mut esi_file = File::open(file_name)?; 
    let mut esi_xml_string = String::new();
    esi_file.read_to_string(&mut esi_xml_string)?;
    let esi = EtherCatInfo::from_xml_str(&esi_xml_string)?;
    let (mut master, domain_idx, offsets) = init_master(&esi, 0_u32)?;
    let cycle_time = Duration::from_micros(50_000);

    loop {
        master.receive()?;
        master.domain(domain_idx).process()?;
       let m_state = master.state()?;
       let d_state = master.domain(domain_idx).state();
        println!("Master state: {:?}", m_state);
        println!("Domain state: {:?}", d_state);
        if m_state.link_up && m_state.al_states == 8 {
            let raw_data = master.domain_data(domain_idx);
            println!("{:?}", raw_data);
        }
        master.domain(domain_idx).queue()?;
        master.send()?;
        thread::sleep(cycle_time);
    }
}

pub fn init_master(
    esi: &EtherCatInfo,
    idx: u32,
) -> Result<
    (
        Master,
        DomainIndex,
        HashMap<PdoEntryIndex, (BitLen, Offset)>,
    ),
    io::Error,
 > 
{
    let mut master = Master::open(idx, MasterAccess::ReadWrite)?;
    master.reserve()?;
    let domain_handle = master.create_domain()?;
    let mut offsets: HashMap<PdoEntryIndex, (u8, Offset)> = HashMap::new();
    for (dev_nr, dev) in esi.description.devices.iter().enumerate() {
        println!("Found device: {}", dev.name);
        let slave_id = SlaveId {
            vendor_id: esi.vendor.id,
            product_code: dev.product_code,
        };
        let rx_entry_indexes: Vec<Vec<PdoEntryIndex>> = dev
            .rx_pdo
            .iter()
            .map(|pdo| {
                pdo.entries
                    .iter()
                    .map(|e| PdoEntryIndex {
                        index: e.index,
                        subindex: e.sub_index.unwrap_or(0) as u8,
                    })
                    .collect()
            })
            .collect();

        let tx_entry_indexes: Vec<Vec<PdoEntryIndex>> = dev
            .tx_pdo
            .iter()
            .map(|pdo| {
                pdo.entries
                    .iter()
                    .map(|e| PdoEntryIndex {
                        index: e.index,
                        subindex: e.sub_index.unwrap_or(0)as u8,
                    })
                    .collect()
            })
            .collect();

        let rx_pdo_entries: Vec<Vec<PdoEntryInfo>> = dev
            .rx_pdo
            .iter()
            .enumerate()
            .map(|(i, pdo)| {
                pdo.entries
                    .iter()
                    .enumerate()
                    .map(|(j, e)| PdoEntryInfo {
                        index: rx_entry_indexes[i][j],
                        bit_length: e.bit_len as u8,
                    })
                    .collect()
            })
            .collect();

        let tx_pdo_entries: Vec<Vec<PdoEntryInfo>> = dev
            .tx_pdo
            .iter()
            .enumerate()
            .map(|(i, pdo)| {
                pdo.entries
                    .iter()
                    .enumerate()
                    .map(|(j, e)| PdoEntryInfo {
                        index: tx_entry_indexes[i][j],
                        bit_length: e.bit_len as u8,
                    })
                    .collect()
            })
            .collect();

        let rx_pdos: Vec<PdoInfo> = dev
            .rx_pdo
            .iter()
            .enumerate()
            .map(|(i, pdo)| PdoInfo {
                index: pdo.index,
                entries: &rx_pdo_entries[i],
            })
            .collect();

        let tx_pdos: Vec<PdoInfo> = dev
            .tx_pdo
            .iter()
            .enumerate()
            .map(|(i, pdo)| PdoInfo {
                index: pdo.index,
                entries: &tx_pdo_entries[i]
            })
            .collect();

        let output = SyncInfo::output(2, &rx_pdos);
        let input = SyncInfo::input(3, &tx_pdos);

        let infos: Vec<_> = vec![
            output,
            input,
        ];

        let mut config = master.configure_slave(SlaveAddr::ByPos(dev_nr as u16), slave_id)?;
        config.config_pdos(&infos)?;

        for pdo in &rx_pdos {
            log::debug!("Positions of RX PDO {:X}:", u16::from(pdo.index));
            for entry in pdo.entries {
                let pos = config.register_pdo_entry(entry.index, domain_handle)?;
                offsets.insert(entry.index, (entry.bit_length, pos));
            }
        }

        for pdo in &tx_pdos {
            log::debug!("Positions of TX PDO {:X}:", u16::from(pdo.index));
            for entry in pdo.entries {
                let offset = config.register_pdo_entry(entry.index, domain_handle)?;
                offsets.insert(entry.index, (entry.bit_length, offset));
            }
        }

        let cfg_index = config.index();
        let cfg_info = master.get_config_info(cfg_index)?;
        log::info!("Config info: {:#?}", cfg_info);
        if cfg_info.slave_position.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to configure slave",
            ));
        }
    }
    Ok((master, domain_handle, offsets))
}

// TODO: de-uglify
fn raw_data_to_pdos(
    data: &[u8],
    offsets: &HashMap<PdoEntryIndex, (BitLen, Offset)>,
) -> HashMap<PdoEntryIndex, Vec<u8>> {
    offsets
        .iter()
        .map(|(pdo_entry_idx, (bit_len, offset))| {
            let start = offset.byte;
            let end = start + {
                if offset.bit != 0 {
                    todo!()
                }
                if bit_len % 8 != 0 {
                    todo!()
                }
                (*bit_len as usize) / 8
            };
            (*pdo_entry_idx, data[start..end].to_vec())
        })
        .collect()
}
