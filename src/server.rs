mod backend;

use clap::Clap;

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

 let cstring = CString::new("".to_owned()).unwrap();

 unsafe {
  GoListJSON(cstring.as_ptr());
 }

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
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_longlong, c_uchar};

#[allow(dead_code, non_snake_case)]
extern "C" {
 fn GoListJSON(path: *const c_char);
 fn GoFetchFiledata(path: *const c_char, startbytepos: c_longlong, endbytepos: c_longlong);
}

#[derive(Debug)]
struct MediaBox {
 size: u64,
 ty: FourCC,
}

impl MediaBox {
 #[allow(dead_code)]
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

/// Receive an array of File entries from Go and insert into turbosql
/// # Safety
/// `json` must be a valid pointer to valid C string until this function returns.
#[no_mangle]
extern "C" fn rust_insert_files_from_go(json: *const c_char) {
 let c_str = unsafe { CStr::from_ptr(json) };
 let string = c_str.to_str().unwrap().to_owned();
 dbg!(string);

 // let mut sender = RESPONSE_TX_CHANNEL.lock().unwrap().clone().unwrap();

 // tokio::spawn(async move {
 //  sender.send(string).await.unwrap();
 // });
}

/// Receive a Filecache entry from Go and insert into turbosql
/// buf is only valid until function return, must be copied
#[no_mangle]
extern "C" fn rust_insert_filecache_from_go(
 json: *const c_char,
 _buf: *const c_uchar,
 _len: c_longlong,
) {
 let c_str = unsafe { CStr::from_ptr(json) };
 let str = c_str.to_str().unwrap();

 dbg!(str);

 // log::info!("rust_insert_filecache_from_go: {:#?}", str);

 // let mut fc: FileCache = serde_json::from_str(str).unwrap();

 // log::info!("rust_insert_filecache_from_go fc: {:#?}", fc);

 // let slice = unsafe { std::slice::from_raw_parts(buf, len as usize) };
 // fc.bytes = Some(slice.to_vec());

 // fc.insert().unwrap();
}
