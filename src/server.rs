mod backend;

use clap::Clap;
use size_format::SizeFormatterBinary as SF;
use std::time::Duration;

#[derive(Clap)]
struct Opts {
 #[clap(short, long)]
 cert_path: Option<String>,
 #[clap(short, long)]
 key_path: Option<String>,
 #[clap(short, long, default_value = "8080")]
 port: u16,
}

#[tokio::main]
async fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
 #[derive(rust_embed::RustEmbed)]
 #[folder = "build"]
 struct Frontend;

 pretty_env_logger::init_timed();
 let opts = Opts::parse();

 log::warn!("warn enabled");
 log::info!("info enabled");
 log::debug!("debug enabled");
 log::trace!("trace enabled");

 //  let _ = std::thread::spawn(|| {
 //   librclone::initialize();
 //   dbg!(librclone::rpc("operations/list", r#"{"fs":"putio:","remote":""}"#)).unwrap();
 //  });

 let sopts = librqbit::session::SessionOptions {
  disable_dht: false,
  disable_dht_persistence: false,
  dht_config: None,
  peer_id: None,
  peer_opts: Some(librqbit::peer_connection::PeerConnectionOptions {
   connect_timeout: Some(Duration::from_secs(10)),
   ..Default::default()
  }),
 };

 let tmp_dir = tempfile::TempDir::new()?;
 let tmp_path = tmp_dir.into_path();

 dbg!(&tmp_path);

 let session = std::sync::Arc::new(
  librqbit::session::Session::new_with_opts(
   tmp_path,
   librqbit::spawn_utils::BlockingSpawner::new(true),
   sopts,
  )
  .await?,
 );

 let torrent_opts = librqbit::session::AddTorrentOptions {
  only_files_regex: None,
  overwrite: true,
  list_only: false,
  force_tracker_interval: None,
  ..Default::default()
 };

 let torrent_path = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.empire-js.us%3A1337&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=wss%3A%2F%2Ftracker.btorrent.xyz&tr=wss%3A%2F%2Ftracker.fastcast.nz&tr=wss%3A%2F%2Ftracker.openwebtorrent.com&ws=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2F&xs=https%3A%2F%2Fwebtorrent.io%2Ftorrents%2Fbig-buck-bunny.torrent";

 let handle = match session.add_torrent(torrent_path.to_string(), Some(torrent_opts)).await? {
  Some(handle) => handle,
  None => return Ok(()),
 };

 librqbit::spawn_utils::spawn("Stats printer", {
  let session = session.clone();
  async move {
   loop {
    session.with_torrents(|torrents| {
              for (idx, torrent) in torrents.iter().enumerate() {
                  match &torrent.state {
                      librqbit::session::ManagedTorrentState::Initializing => {
                          log::info!("[{}] initializing", idx);
                      },
                      librqbit::session::ManagedTorrentState::Running(handle) => {
                          let peer_stats = handle.torrent_state().peer_stats_snapshot();
                          let stats = handle.torrent_state().stats_snapshot();
                          let speed = handle.speed_estimator();
                          let total = stats.total_bytes;
                          let progress = stats.total_bytes - stats.remaining_bytes;
                          let downloaded_pct = if stats.remaining_bytes == 0 {
                              100f64
                          } else {
                              (progress as f64 / total as f64) * 100f64
                          };
                          log::info!(
                              "[{}]: {:.2}% ({:.2}), down speed {:.2} Mbps, fetched {}, remaining {:.2} of {:.2}, uploaded {:.2}, peers: {{live: {}, connecting: {}, queued: {}, seen: {}}}",
                              idx,
                              downloaded_pct,
                              SF::new(progress),
                              speed.download_mbps(),
                              SF::new(stats.fetched_bytes),
                              SF::new(stats.remaining_bytes),
                              SF::new(total),
                              SF::new(stats.uploaded_bytes),
                              peer_stats.live,
                              peer_stats.connecting,
                              peer_stats.queued,
                              peer_stats.seen,
                          );
                      },
                  }
              }
          });
    tokio::time::sleep(Duration::from_secs(1)).await;
   }
  }
 });

 handle.wait_until_completed().await?;

 match (opts.key_path, opts.cert_path) {
  (Some(key_path), Some(cert_path)) => {
   eprintln!("Serving HTTPS on port {}", opts.port);
   warp::serve(turbocharger::warp_routes(Frontend))
    .tls()
    .cert_path(cert_path)
    .key_path(key_path)
    .run(([0, 0, 0, 0], opts.port))
    .await;
  }
  (None, None) => {
   eprintln!("Serving (unsecured) HTTP on port {}", opts.port);
   opener::open(format!("http://127.0.0.1:{}", opts.port)).ok();
   warp::serve(turbocharger::warp_routes(Frontend)).run(([0, 0, 0, 0], opts.port)).await;
  }
  _ => eprintln!("Both key-path and cert-path must be specified for HTTPS."),
 }

 Ok(())
}

use four_cc::FourCC;
use nom::bytes::streaming::take;
use nom::number::streaming::{be_u32, be_u64};
use nom::IResult;
use std::convert::TryInto;

#[derive(Debug)]
struct MediaBox {
 size: u64,
 ty: FourCC,
}

impl MediaBox {
 fn parse(i: &[u8]) -> IResult<&[u8], MediaBox> {
  let (i, size) = be_u32(i)?;
  let (i, ty) = take(4usize)(i)?;
  let ty: FourCC = ty.try_into().unwrap();
  let (i, size) = match size {
   1 => be_u64(i)?,
   _ => (i, size.into()),
  };

  Ok((i, MediaBox { size, ty }))
 }
}
