use sunk::Sunk;
use song::Song;
use error::*;
use json;
use macros::*;

#[derive(Debug)]
pub struct Playlist {
    id: u64,
    name: String,
    song_count: u64,
    duration: u64,
    cover: String,
}

impl Playlist {
    /// Parses a JSON map into a Playlist struct.
    pub fn from(j: &json::Value) -> Result<Playlist> {
        if !j.is_object() { return Err(Error::ParseError("not an object")) }

        Ok(Playlist {
            id: fetch!(j->id: as_str, u64),
            name: fetch!(j->name: as_str).into(),
            song_count: fetch!(j->songCount: as_u64),
            duration: fetch!(j->duration: as_u64),
            cover: fetch!(j->coverArt: as_str).into(),
        })
    }

    /// Fetches the songs contained in a playlist.
    fn songs(&self, sunk: &mut Sunk) -> Result<Vec<Song>> {
        get_playlist_content(sunk, self.id)
    }
}

fn get_playlists(sunk: &mut Sunk, user: Option<String>) -> Result<Vec<Playlist>> {
    let arg = if let Some(u) = user {
        vec![("username", u)]
    } else {
        vec![]
    };
    let (_, res) = sunk.get("getPlaylists", arg)?;
    let mut pls = vec![];
    for pl in pointer!(res, "/subsonic-response/playlists/playlist")
        .as_array().ok_or(Error::ParseError("not an array"))?
    {
        pls.push(Playlist::from(pl)?);
    }
    Ok(pls)
}

fn get_playlist(sunk: &mut Sunk, id: u64) -> Result<Playlist> {
    let (_, res) = sunk.get("getPlaylist", vec![("id", id)])?;
    Playlist::from(&res["subsonic-response"]["playlist"])
}

fn get_playlist_content(sunk: &mut Sunk, id: u64) -> Result<Vec<Song>> {
    let (_, res) = sunk.get("getPlaylist", vec![("id", id)])?;
    let mut list = vec![];
    for song in pointer!(res, "/subsonic-response/playlist/entry")
        .as_array().ok_or(Error::ParseError("not an array"))?
    {
        list.push(Song::from(song)?);
    }
    Ok(list)
}

/// Creates a playlist with the given name.
///
/// Since API version 1.14.0, the newly created playlist is returned. In earlier
/// versions, an empty response is returned.
fn create_playlist(sunk: &mut Sunk, name: String, songs: Option<Vec<u64>>) ->
    Result<Option<Playlist>>
{
    let mut args = vec![("name", name)];
    if let Some(songs) = songs {
        for id in songs {
            // `to_string()`, otherwise we have type conflicts.
            args.push(("songId", id.to_string()))
        }
    }
    let (_, res) = sunk.get("createPlaylist", args)?;
    // TODO Match the API and return the playlist on new versions.

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    #[test]
    fn test_songs_from_playlist() {
        let raw = json!(
            {
                "id" : "1",
                "name" : "Sleep Hits",
                "owner" : "user",
                "public" : false,
                "songCount" : 32,
                "duration" : 8334,
                "created" : "2018-01-01T14:45:07.464Z",
                "changed" : "2018-01-01T14:45:07.478Z",
                "coverArt" : "pl-2"
            }
        );

        let parsed = Playlist::from(raw).unwrap();
        let auth = load_credentials().unwrap();
        let mut srv = Sunk::new(&auth.0, &auth.1, &auth.2).unwrap();
        let songs = parsed.songs(&mut srv).unwrap();
    }
}
