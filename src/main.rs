use std::{
    fs::File,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sysinfo::{ComponentExt, DiskExt, NetworkExt, System, SystemExt};

use std::io::{stdout, Write};

use clap::Parser;

mod tgph_format;
use tgph_format::TGPH;

/// Gather data about system state
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Where to save the data, if file already exists start appending
    output_path: String,

    /// How many entries per container are allowed
    entry_limit: usize,

    /// How many seconds between each system state read
    timeout_period: u64,
}

fn main() {
    let args = Args::parse();

    let mut sys = System::new_all();
    let mut stdout = stdout();

    let mut tgph = match File::open(args.output_path.clone()) {
        Ok(mut file) => {
            let mut res = TGPH::deserialize_from(&mut file).unwrap();
            res.entry_limit = args.entry_limit;
            res
        }
        Err(_) => TGPH::new(args.entry_limit),
    };

    let mut points_saved = tgph.containers.len();

    loop {
        sys.refresh_all();

        tgph.append(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            "Unix timestamp",
        );

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
            tgph.append(
                component.temperature(),
                &format!("{} Temperature [C]", component.label()),
            );
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

        let mut output_file = File::create(args.output_path.clone()).unwrap();
        tgph.serialize_into(&mut output_file).unwrap();

        points_saved += 1;

        print!("\rSaved {points_saved} snapshots");
        stdout.flush().unwrap();

        sleep(Duration::from_secs(args.timeout_period));
    }
}
