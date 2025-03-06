use chrono::DateTime;
use chrono::Datelike;
use clap::Parser;
use filetime::FileTime;
use std::fmt::Display;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
enum Error<P = PathBuf> {
	#[error("Walkdir error: {0}")]
	WalkDir(#[from] walkdir::Error),

	#[error("IO error: {0}")]
	Io(#[from] io::Error),

	#[error("Skipping file: {0}")]
	Skipping(P),

	#[error("File has no name")]
	NoName(P),

	#[error("DateTime error")]
	DateTime(P),

	#[error("Mime error")]
	Mime(P),

	#[error("{0} is a directory")]
	Dir(P),
}

#[derive(Parser)]
struct Cli {
	#[arg(short, long, required = true)]
	source: PathBuf,

	#[arg(short, long, required = true)]
	destination: PathBuf,

	#[arg(short, long, required = true)]
	person: String,

	#[arg(short = 'y', long, default_value = "false")]
	dry_run: bool,
}

struct Target {
	abs_path: PathBuf,
	extension: Extension,
	mtime: MTime,
	name: String,
}

impl TryFrom<&Path> for Target {
	type Error = Error;

	fn try_from(path: &Path) -> Result<Self> {
		if path.is_dir() {
			return Err(Error::Dir(path.to_path_buf()));
		}

		let abs_path = std::fs::canonicalize(path)?;
		let extension = Extension::try_from(&abs_path)?;
		let name = abs_path
			.file_name()
			.ok_or(Error::NoName(path.to_path_buf()))?
			.to_str()
			.ok_or(Error::NoName(path.to_path_buf()))?
			.to_string();
		let mtime = MTime::try_from(&abs_path)?;

		Ok(Self { abs_path, name, extension, mtime })
	}
}

struct MTime {
	year: String,
	month: String,
}

impl TryFrom<&PathBuf> for MTime {
	type Error = Error;

	fn try_from(path: &PathBuf) -> Result<Self> {
		let metadata = std::fs::metadata(path)?;
		let filetime = FileTime::from_last_modification_time(&metadata);
		let secs = filetime.seconds();
		let date = DateTime::from_timestamp(secs, 0).ok_or(Error::DateTime(path.clone()))?;
		let month = date.format("%B").to_string().to_lowercase();
		let year = date.year().to_string();

		Ok(Self { year, month })
	}
}

#[derive(Clone)]
enum Extension {
	Image,
	Video,
}

impl TryFrom<&PathBuf> for Extension {
	type Error = Error;

	fn try_from(path: &PathBuf) -> Result<Self> {
		let extension = path
			.extension()
			.ok_or(Error::Skipping(path.clone()))?
			.to_str()
			.ok_or(Error::Skipping(path.clone()))?
			.to_lowercase();

		let mime = match extension {
			e if e == "arw" => "image".to_string(),
			e if e == "heic" => "image".to_string(),
			_ => mime_guess::from_ext(&extension)
				.first()
				.ok_or(Error::Mime(path.clone()))?
				.to_string(),
		};

		let extension = match mime {
			ext if ext.starts_with("image") => Extension::Image,
			ext if ext.starts_with("video") => Extension::Video,
			_ => return Err(Error::Skipping(path.to_owned())),
		};

		Ok(extension)
	}
}

impl Display for Extension {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let w = match self {
			Extension::Video => "videos",
			Extension::Image => "pictures",
		};

		write!(f, "{}", w)
	}
}

fn main() -> Result<()> {
	let mut cli = Cli::parse();
	cli.destination = std::fs::canonicalize(&cli.destination)?;

	for item in WalkDir::new(cli.source) {
		let item = item?;
		let path = item.path();

		let target = match Target::try_from(path) {
			Ok(target) => target,
			Err(err) => {
				eprintln!("Error: {:#?}", err);
				continue;
			}
		};

		let destination = cli
			.destination
			.join(target.extension.to_string())
			.join(&cli.person)
			.join(&target.mtime.year)
			.join(&target.mtime.month);

		// match cli.dry_run {
		// 	false => {
		let dest_dir = &destination;
		let dest_file = &destination.join(&target.name);

		match std::fs::create_dir_all(dest_dir) {
			Ok(_) => (),
			Err(_) => println!("Directory {} already created!", &dest_dir.display()),
		};

		match std::fs::rename(&target.abs_path, dest_file) {
			Ok(_) => println!("{} -> {}", target.abs_path.display(), dest_file.display()),
			Err(_) => println!("File {} already exists!", &dest_file.display()),
		};
		// 	}
		// 	true => (),
		// }
	}

	Ok(())
}
