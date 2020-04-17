use ethercat as ec;
use std::{thread, time::Duration};

pub fn main() -> Result<(), std::io::Error> {
    let mut master = ec::Master::reserve(0)?;
    let domain_handle = master.create_domain()?;

    // ID of Weidmüller UR20 I/O coupler
    let slave_id = ec::SlaveId {
        vendor_id: 0x230,
        product_code: 0x4f911c30,
    };

    let rx_entry_indexes: Vec<Vec<_>> = vec![
        // Indexes of digital coupler outputs
        (0..=15)
            .into_iter()
            .map(|i| ec::PdoEntryIndex {
                index: 0xF200,
                subindex: i + 1,
            })
            .collect(),
        // Indexes of digital outputs 0-3 (UR20-RO-CO-255)
        (0..=4)
            .into_iter()
            .map(|i| match i {
                0..=3 => ec::PdoEntryIndex {
                    index: 0x7000,
                    subindex: i + 1,
                },
                4 => ec::PdoEntryIndex {
                    index: 0x0000,
                    subindex: 0,
                },
                _ => unreachable!(),
            })
            .collect(),
        // Indexes of digital outputs 0-15 (UR20-16DO-P)
        (0..=15)
            .into_iter()
            .map(|i| ec::PdoEntryIndex {
                index: 0x7010,
                subindex: i + 1,
            })
            .collect(),
    ];

    let tx_entry_indexes: Vec<Vec<_>> = vec![
        // Indexes of digital coupler inputs
        (0..=15)
            .into_iter()
            .map(|i| ec::PdoEntryIndex {
                index: 0xF100,
                subindex: i + 1,
            })
            .collect(),
        // Index of module state (UR20-RO-CO-255)
        vec![ec::PdoEntryIndex {
            index: 0x6000,
            subindex: 1,
        }],
        // Index of module state (UR20-16DO-P)
        vec![ec::PdoEntryIndex {
            index: 0x6010,
            subindex: 1,
        }],
    ];

    let rx_pdo_entries: Vec<Vec<_>> = vec![
        // PDO entries of coupler outputs
        (0..=15)
            .into_iter()
            .map(|i| ec::PdoEntryInfo {
                index: rx_entry_indexes[0][i],
                bit_length: 1,
            })
            .collect(),
        // PDO entries of digital output 0 and 3
        (0..=4)
            .into_iter()
            .map(|i| match i {
                0..=3 => ec::PdoEntryInfo {
                    index: rx_entry_indexes[1][i],
                    bit_length: 1,
                },
                4 => ec::PdoEntryInfo {
                    index: rx_entry_indexes[1][i],
                    bit_length: 4,
                },
                _ => unreachable!(),
            })
            .collect(),
        // PDO entries of digital out 0..15  (UR20-16DO-P)
        (0..=15)
            .into_iter()
            .map(|i| ec::PdoEntryInfo {
                index: rx_entry_indexes[2][i],
                bit_length: 1,
            })
            .collect(),
    ];

    // PDO entry of module state
    let tx_pdo_entries: Vec<Vec<_>> = vec![
        (0..=15)
            .into_iter()
            .map(|i| ec::PdoEntryInfo {
                index: tx_entry_indexes[0][i],
                bit_length: 1,
            })
            .collect(),
        (0..=0)
            .into_iter()
            .map(|i| ec::PdoEntryInfo {
                index: tx_entry_indexes[1][i],
                bit_length: 8,
            })
            .collect(),
        (0..=0)
            .into_iter()
            .map(|i| ec::PdoEntryInfo {
                index: tx_entry_indexes[2][i],
                bit_length: 8,
            })
            .collect(),
    ];

    // RX PDOs of outputs
    let rx_pdos = vec![
        ec::PdoInfo {
            index: 0x16FF,
            entries: &rx_pdo_entries[0],
        },
        ec::PdoInfo {
            index: 0x1600,
            entries: &rx_pdo_entries[1],
        },
        ec::PdoInfo {
            index: 0x1601,
            entries: &rx_pdo_entries[2],
        },
    ];

    // TX PDOs of inputs
    let tx_pdos = vec![
        ec::PdoInfo {
            index: 0x1AFF,
            entries: &tx_pdo_entries[0],
        },
        ec::PdoInfo {
            index: 0x1A00,
            entries: &tx_pdo_entries[1],
        },
        ec::PdoInfo {
            index: 0x1A01,
            entries: &tx_pdo_entries[2],
        },
    ];

    // Sync masters
    let infos = vec![
        ec::SyncInfo::output(2, &rx_pdos),
        ec::SyncInfo::input(3, &tx_pdos),
    ];

    let mut config = master.configure_slave(ec::SlaveAddr::ByPos(0), slave_id)?;
    config.config_pdos(&infos)?;

    for pdos in &rx_entry_indexes {
        for pdo in pdos {
            let pos = config.register_pdo_entry(*pdo, domain_handle)?;
            println!(
                "Position of RX entry {:X}.{} is {:?}",
                pdo.index, pdo.subindex, pos
            );
        }
    }

    for pdos in &tx_entry_indexes {
        for pdo in pdos {
            let pos = config.register_pdo_entry(*pdo, domain_handle)?;
            println!(
                "Position of TX entry {:X}.{} is {:?}",
                pdo.index, pdo.subindex, pos
            );
        }
    }

    let cfg_index = config.index();
    let cfg_info = master.get_config_info(cfg_index)?;
    println!("Config info: {:#?}", cfg_info);
    if cfg_info.slave_position.is_none() {
        panic!("Unable to configure slave");
    }

    let info = master.get_info();
    println!("EtherCAT master: {:#?}", info);

    println!("Activate master");
    master.activate()?;

    loop {
        master.receive()?;
        master.domain(domain_handle).process()?;

        println!("Master state: {:?}", master.state());
        println!("Domain state: {:?}", master.domain(domain_handle).state());

        let data = master.domain_data(domain_handle);
        println!("Received data: {:?}", data);

        // Toggle output 3 (bit 1)
        data[2] ^= 0b_0000_0010;

        master.domain(domain_handle).queue()?;
        master.send()?;

        thread::sleep(Duration::from_millis(50));
    }
}
