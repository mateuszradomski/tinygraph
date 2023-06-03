use std::{
    fs::File,
    io::{Cursor, Read},
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sysinfo::{ComponentExt, DiskExt, NetworkExt, System, SystemExt, CpuExt};

use std::io::{stdout, Write};

use clap::Parser;

use libdeflater::{CompressionLvl, Compressor, Decompressor};

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

fn decompress<R: Read>(stream: &mut R) -> Result<Vec<u8>, std::io::Error> {
    let mut gz_data = Vec::new();
    stream.read_to_end(&mut gz_data)?;

    if gz_data.len() < 10 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "gz data is too short (for magic bytes + footer",
        ));
    }

    let isize = {
        let isize_start = gz_data.len() - 4;
        let isize_bytes = &gz_data[isize_start..];
        let mut ret: u32 = isize_bytes[0] as u32;
        ret |= (isize_bytes[1] as u32) << 8;
        ret |= (isize_bytes[2] as u32) << 16;
        ret |= (isize_bytes[3] as u32) << 24;
        ret as usize
    };

    let mut decompressor = Decompressor::new();
    let mut outbuf = Vec::new();
    outbuf.resize(isize, 0);
    decompressor.gzip_decompress(&gz_data, &mut outbuf).unwrap();
    Ok(outbuf)
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut sys = System::new_all();
    let mut stdout = stdout();

    if !args.output_path.ends_with(".gz") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected the output file to be a gzip file with \".gz\" ending",
        ));
    }

    let mut tgph = match File::open(args.output_path.clone()) {
        Ok(mut compressed_file) => {
            let decompressed = decompress(&mut compressed_file).unwrap();
            let mut cursor = Cursor::new(decompressed);

            let mut res = TGPH::deserialize_from(&mut cursor).unwrap();
            res.entry_limit = args.entry_limit;
            res
        }
        Err(_) => TGPH::new(args.entry_limit),
    };

    let mut points_saved = if tgph.containers.len() == 0 {
        0
    } else {
        match &tgph.containers[0].elements {
            tgph_format::ElementArrayType::U32(v) => v.len(),
            tgph_format::ElementArrayType::FLOAT32(v) => v.len(),
            tgph_format::ElementArrayType::STRING(v) => v.len(),
        }
    };

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
                &format!("Disk {} Available Space [GB]", disk.name().to_str().unwrap()),
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

        for (i, cpu) in sys.cpus().iter().enumerate() {
            tgph.append(cpu.cpu_usage(), &format!("CPU {} Usage [%]", i))
        }

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

        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        let compressed_data = {
            let mut compressor = Compressor::new(CompressionLvl::default());
            let max_sz = compressor.gzip_compress_bound(output_buffer.len());
            let mut compressed_data = Vec::new();
            compressed_data.resize(max_sz, 0);
            let actual_sz = compressor
                .gzip_compress(&output_buffer, &mut compressed_data)
                .unwrap();
            compressed_data.resize(actual_sz, 0);
            compressed_data
        };

        let mut output_file = File::create(args.output_path.clone())?;
        output_file.write_all(&compressed_data)?;

        points_saved += 1;

        print!("\rSaved {points_saved} snapshots");
        stdout.flush().unwrap();

        sleep(Duration::from_secs(args.timeout_period));
    }
}
