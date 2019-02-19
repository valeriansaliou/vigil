// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs::File;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use time::{self, Duration};

use rocket::http::hyper::header::{CacheControl, CacheDirective, Expires, HttpDate};
use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder};

const ASSETS_EXPIRE_SECONDS: u32 = 10800;

#[derive(Debug)]
pub struct AssetFile(PathBuf, File);

// Notice: this is a re-implementation of Rocket native NamedFile, with more response headers
// See: https://api.rocket.rs/src/rocket/response/named_file.rs.html
impl AssetFile {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<AssetFile> {
        let file = File::open(path.as_ref())?;
        Ok(AssetFile(path.as_ref().to_path_buf(), file))
    }

    #[inline(always)]
    pub fn file(&self) -> &File {
        &self.1
    }

    #[inline(always)]
    pub fn take_file(self) -> File {
        self.1
    }

    #[inline(always)]
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.1
    }

    #[inline(always)]
    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

impl<'r> Responder<'r> for AssetFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let mut response = self.1.respond_to(req)?;

        // Set cache headers
        response.set_header(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(ASSETS_EXPIRE_SECONDS),
        ]));

        response.set_header(Expires(HttpDate(
            time::now() + Duration::seconds(ASSETS_EXPIRE_SECONDS as i64),
        )));

        // Set content type header?
        if let Some(ext) = self.0.extension() {
            if let Some(ct) = ContentType::from_extension(&ext.to_string_lossy()) {
                response.set_header(ct);
            }
        }

        Ok(response)
    }
}

impl Deref for AssetFile {
    type Target = File;

    fn deref(&self) -> &File {
        &self.1
    }
}

impl DerefMut for AssetFile {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.1
    }
}

impl io::Read for AssetFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file().read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.file().read_to_end(buf)
    }
}

impl io::Write for AssetFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file().flush()
    }
}

impl io::Seek for AssetFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.file().seek(pos)
    }
}

impl<'a> io::Read for &'a AssetFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file().read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.file().read_to_end(buf)
    }
}

impl<'a> io::Write for &'a AssetFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file().flush()
    }
}

impl<'a> io::Seek for &'a AssetFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.file().seek(pos)
    }
}
