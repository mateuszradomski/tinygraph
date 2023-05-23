use std::{thread::sleep, time::{SystemTime, UNIX_EPOCH}};

use sysinfo::{ComponentExt, DiskExt, NetworkExt, System, SystemExt};

use std::io::{stdout, Write};

mod tgph_format;
use tgph_format::TGPH;

fn main() {
    let mut points_saved = 0;
    let mut sys = System::new_all();
    let mut stdout = stdout();

    let mut tgph = TGPH::default();
    loop {
        sys.refresh_all();

        tgph.append(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32, "Unix timestamp");

        for disk in sys.disks() {
            tgph.append(
                (disk.total_space() / 1024 / 1024 / 1024) as u32,
                &format!("Disk {} Total Space [GB]", disk.name().to_str().unwrap()),
            );
            tgph.append(
                (disk.available_space() / 1024 / 1024 / 1024) as u32,
                &format!("Disk {} Avaiable Space [GB]", disk.name().to_str().unwrap()),
            );
        }

        for (interface_name, data) in sys.networks() {
            tgph.append(
                data.received() as u32,
                &format!("Interface {} Received [bytes]", interface_name),
            );
            tgph.append(
                data.transmitted() as u32,
                &format!("Interface {} Transmitted [bytes]", interface_name),
            );
        }

        for component in sys.components() {
            tgph.append(component.temperature(), &format!("{} Temperature [C]", component.label()));
        }

        tgph.append(sys.cpus().len() as u32, "CPU Count");

        tgph.append(
            (sys.total_memory() / 1024 / 1024) as u32,
            "Total memory [MB]",
        );
        tgph.append((sys.used_memory() / 1024 / 1024) as u32, "Used memory [MB]");
        tgph.append((sys.total_swap() / 1024 / 1024) as u32, "Total swap [MB]");
        tgph.append((sys.used_swap() / 1024 / 1024) as u32, "Used swap [MB]");

        tgph.append(
            sys.kernel_version()
                .unwrap_or("UNDEFINED".to_string())
                .to_string(),
            "Kernel Version",
        );
        tgph.append(
            sys.os_version()
                .unwrap_or("UNDEFINED".to_string())
                .to_string(),
            "OS Version",
        );
        tgph.append(
            sys.host_name()
                .unwrap_or("UNDEFINED".to_string())
                .to_string(),
            "Hostname",
        );

        let mut output_file = std::fs::File::create("data.tgph").unwrap();
        tgph.serialize_into(&mut output_file).unwrap();

        points_saved += 1;

        print!("\rSaved {points_saved} snapshots");
        stdout.flush().unwrap();

        sleep(std::time::Duration::from_secs(15));
    }
}
