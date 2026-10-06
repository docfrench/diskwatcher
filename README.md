<div align="left">

[![CI](https://github.com/docfrench/diskwatcher/actions/workflows/ci.yml/badge.svg)](https://github.com/docfrench/diskwatcher/actions/workflows/ci.yml)
![Rust edition](https://img.shields.io/badge/rust-2024_edition-orange)

</div>

# diskwatcher

A small Rust service that collects Unraid array state and per-disk SMART data and formats it as Prometheus metrics. Built for my Unraid server as part of a Prometheus + Grafana homelab monitoring stack.

> **Status:** work in progress. Collection and formatting are done; metrics are currently written to stdout on each scan. Serving them on an HTTP `/metrics` endpoint for Prometheus to scrape is the next step.

## Features

- Reads array and disk state directly from Unraid's `emhttp` state files (`var.ini`, `disks.ini`)
- Collects SMART attributes with `smartctl`, with separate parsers for SATA/SAS drives and NVMe drives
- Skips SMART reads on spun-down HDDs (`smartctl -n standby`) so scans don't wake the array
- Emits metrics in Prometheus text exposition format, labeled by device and disk type
- Metrics with no value (missing attribute, disk in standby) are omitted rather than reported as zero
- No runtime dependencies beyond the Rust standard library and `smartmontools`

## Quick start

```bash
docker run -d --name diskwatcher \
  --restart unless-stopped \
  --privileged \
  -v /dev:/dev \
  -v /var/local/emhttp:/var/local/emhttp:ro \
  ghcr.io/docfrench/diskwatcher:latest
```

Then watch a scan cycle:

```bash
docker logs -f diskwatcher
```

> `smartctl` needs raw device access, so the container runs `--privileged` with `/dev` mounted. That gives it broad access to the host's hardware; only run it on a host you control. The `emhttp` directory is mounted read-only.

## How it works

Every 60 seconds (hardcoded), diskwatcher runs a scan:

1. Parses `/var/local/emhttp/var.ini` for array state: started/stopped, resync progress, sync errors, and disabled/invalid/missing disk counts.
2. Parses `/var/local/emhttp/disks.ini` for each disk's device, type (Parity, Data, Cache, Flash), status, temperature, error count, read/write counters, and size.
3. Runs `smartctl -A` against each device. Devices with `nvme` in the name are parsed as NVMe health logs; everything else is parsed as an ATA SMART attribute table.
4. Formats everything as Prometheus metrics and prints it.

Example output (values illustrative):

```text
disk_temperature_celsius{device="sdg",disk_type="Parity"} 34
disk_errors_total{device="sdg",disk_type="Parity"} 0
disk_size_bytes{device="sdg",disk_type="Parity"} 12000138625024
disk_smart_hdd_reallocated_sector_ct{device="sdg",disk_type="Parity"} 0
disk_smart_hdd_power_on_hours{device="sdg",disk_type="Parity"} 21877
disk_ok{device="sdg",disk_type="Parity"} 1
disk_smart_ssd_percentage_used{device="nvme0n1",disk_type="Cache"} 4
array_started 1
array_sync_errors 0
array_disks_disabled 0
array_info{md_state="STARTED",fs_state="Started",resync_action="check P"} 1
```

## Metrics

| Metric | Description |
|--------|-------------|
| `array_started` | 1 if the array is started, 0 otherwise |
| `array_resync_position`, `array_resync_size` | Parity check / rebuild progress |
| `array_resync_corrections`, `array_sync_errors` | Corrections and sync errors from the last check |
| `array_disks`, `array_disks_disabled`, `array_disks_invalid`, `array_disks_missing` | Array disk counts |
| `array_info` | Always 1; array, filesystem, and resync state carried as labels |
| `disk_ok` | 1 if Unraid reports the disk as `DISK_OK`, 0 otherwise |
| `disk_temperature_celsius` | Temperature as reported by Unraid |
| `disk_errors_total`, `disk_reads_total`, `disk_writes_total` | Unraid's per-disk counters |
| `disk_size_bytes` | Disk size |
| `disk_smart_hdd_*` | ATA SMART raw values (reallocated/pending sectors, CRC errors, power-on hours, helium level, etc.) |
| `disk_smart_ssd_*` | NVMe health log values (percentage used, available spare, media errors, data units read/written, etc.) |

Every `disk_*` metric carries `device` and `disk_type` labels.

## Roadmap

- [x] Parse Unraid array and disk state
- [x] SMART collection for HDDs and NVMe drives
- [x] Prometheus exposition formatting
- [x] CI/CD pipeline with deployment to Unraid
- [ ] Serve metrics on an HTTP `/metrics` endpoint
- [ ] Add `# HELP` / `# TYPE` metadata
- [ ] Prometheus scrape config and Grafana dashboard

## CI/CD

Every push to `main` runs a three-stage GitHub Actions pipeline:

1. **Lint:** `cargo clippy --all-targets --all-features` with warnings treated as errors
2. **Build and push:** Docker image to GHCR, tagged with the commit SHA and `latest`
3. **Deploy:** a self-hosted runner on the Unraid host pulls the new image and replaces the running container

Pull requests run the lint stage only.

## Development

```bash
cargo build --release
RUSTFLAGS="-Dwarnings" cargo clippy --all-targets --all-features
```

Requires Rust 1.85+ (2024 edition). Running it outside a container requires an Unraid host with `smartmontools` installed and root access to the disk devices.

## License

MIT. See [LICENSE](LICENSE).
