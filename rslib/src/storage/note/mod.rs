// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    err::Result,
    notes::{Note, NoteID},
    tags::{join_tags, split_tags},
};
use rusqlite::{params, OptionalExtension};

fn split_fields(fields: &str) -> Vec<String> {
    fields.split('\x1f').map(Into::into).collect()
}

fn join_fields(fields: &[String]) -> String {
    fields.join("\x1f")
}

impl super::SqliteStorage {
    pub fn get_note(&self, nid: NoteID) -> Result<Option<Note>> {
        let mut stmt = self.db.prepare_cached(include_str!("get.sql"))?;
        stmt.query_row(params![nid], |row| {
            Ok(Note {
                id: nid,
                guid: row.get(0)?,
                ntid: row.get(1)?,
                mtime: row.get(2)?,
                usn: row.get(3)?,
                tags: split_tags(row.get_raw(4).as_str()?)
                    .map(Into::into)
                    .collect(),
                fields: split_fields(row.get_raw(5).as_str()?),
                sort_field: None,
                checksum: None,
            })
        })
        .optional()
        .map_err(Into::into)
    }

    /// Caller must call note.prepare_for_update() prior to calling this.
    pub(crate) fn update_note(&self, note: &Note) -> Result<()> {
        let mut stmt = self.db.prepare_cached(include_str!("update.sql"))?;
        stmt.execute(params![
            note.guid,
            note.ntid,
            note.mtime,
            note.usn,
            join_tags(&note.tags),
            join_fields(&note.fields),
            note.sort_field.as_ref().unwrap(),
            note.checksum.unwrap(),
            note.id
        ])?;
        Ok(())
    }
}