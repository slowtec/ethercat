use ethercat::{Master, MasterAccess, SdoEntryAddr};

pub fn main() -> Result<(), std::io::Error> {
    let mut master = Master::open(0, MasterAccess::ReadOnly)?;
    let slave_pos = 0;
    let slave = master.get_slave_info(slave_pos)?;
    for i in 0..slave.sdo_count {
        let sdo_info = master.get_sdo(slave_pos, i)?;
        for j in 0..sdo_info.max_subindex + 1 {
            let addr = SdoEntryAddr::Index(sdo_info.index, j);
            let entry = master.get_sdo_entry(slave_pos, addr)?;
            println!("{:#?}", entry);
        }
    }
    Ok(())
}
