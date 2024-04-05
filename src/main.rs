use chrono::DateTime;
use chrono::Datelike;
use clap::Parser;
use filetime::FileTime;
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
	dry_run: Option<bool>,
}

impl AsRef<Path> for Cli {
	fn as_ref(&self) -> &Path {
		&self.destination
	}
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
			.ok_or(Error::Skipping(path.clone()))?;

		let mime = mime_guess::from_ext(extension)
			.first()
			.ok_or(Error::Mime(path.clone()))?
			.to_string();

		let extension = match mime {
			ext if ext.starts_with("image") => Extension::Image,
			ext if ext.starts_with("video") => Extension::Video,
			_ => return Err(Error::Skipping(path.clone())),
		};

		Ok(extension)
	}
}

impl ToString for Extension {
	fn to_string(&self) -> String {
		match self {
			Extension::Video => "videos".to_owned(),
			Extension::Image => "pictures".to_owned(),
		}
	}
}

fn main() -> Result<()> {
	let mut cli = Cli::parse();
	cli.destination = std::fs::canonicalize(&cli)?;

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
			.join(target.mtime.year)
			.join(target.mtime.month)
			.join(target.name);

		if !cli.dry_run.unwrap() {
			// TODO: move files
		}

		println!("{} -> {}", target.abs_path.display(), destination.display());
	}

	Ok(())
}
