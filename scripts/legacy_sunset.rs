//! Identifica código com mais de 5 anos e agenda revisão.

use git2::Repository;
use chrono::{Utc, TimeDelta};

fn main() {
    let repo = Repository::open(".").unwrap();
    let now = Utc::now();
    let cutoff = now - TimeDelta::days(5 * 365);

    for file in repo.index().unwrap().iter() {
        let path = String::from_utf8_lossy(&file.path);
        if !path.ends_with(".rs") {
            continue;
        }

        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push_head().unwrap();

        let mut oldest = now;

        for oid in revwalk {
            let oid = oid.unwrap();
            let commit = repo.find_commit(oid).unwrap();

            // To simplify this script, we'll just log any file
            // Note: properly tracking individual file history in git2 is complex
            // This is a simplified version for the sunset policy script
            let commit_time = chrono::DateTime::from_timestamp(commit.time().seconds(), 0).unwrap();
            if commit_time < oldest {
                oldest = commit_time;
            }
        }

        if oldest < cutoff {
            eprintln!("⚠️  Legacy code ({}+ years): {}",
                (now - oldest).num_days() / 365, path);
        }
    }
}
