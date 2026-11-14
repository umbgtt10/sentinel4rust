// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::world::World;
use sentinel4rust::model::github::discovered_repo::DiscoveredRepo;
use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::onboarding::toml_roster::TomlRoster;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::discovery_refresh::DiscoveryRefresh;
use sentinel4rust::traits::repo_discovery::RepoDiscovery;
use std::rc::Rc;

const START: i64 = 1_792_800_000;

fn discovered(name: &str) -> DiscoveredRepo {
    DiscoveredRepo {
        name: name.to_string(),
        url: format!("https://github.com/o/{name}.git"),
        branch: String::from("main"),
    }
}

fn names(repos: &[OnboardedRepo]) -> Vec<&str> {
    repos.iter().map(|repo| repo.name.as_str()).collect()
}

fn refresh(world: &Rc<World>, discovery: bool) -> DiscoveryRefresh {
    let roster = TomlRoster
        .parse("work_dir = \"/w\"\nresults_url = \"r\"\npause_seconds = 300\ndiscovery_minutes = 60\n[[repo]]\nname = \"a\"\nurl = \"u\"\n")
        .unwrap();
    let port: Option<Rc<dyn RepoDiscovery>> = if discovery { Some(world.clone()) } else { None };
    DiscoveryRefresh::new(&roster, port, world.clone())
}

#[test]
fn refresh_before_it_is_due_returns_nothing() {
    // Arrange
    let world = Rc::new(World::default());
    let mut refresh = refresh(&world, true);
    world.queue_discovery(Ok(vec![discovered("b")]));
    refresh.refresh(Timestamp::from_unix(START));

    // Act
    let refreshed = refresh.refresh(Timestamp::from_unix(START + 3599));

    // Assert
    assert!(!refreshed);
    assert_eq!(refresh.due(), Some(Timestamp::from_unix(START + 3600)));
}

#[test]
fn refresh_failing_keeps_the_list_logs_and_retries_after_the_pause() {
    // Arrange
    let world = Rc::new(World::default());
    let mut refresh = refresh(&world, true);
    world.queue_discovery(Err(DiscoveryError(String::from("curl: exited with 6"))));

    // Act
    let refreshed = refresh.refresh(Timestamp::from_unix(START));

    // Assert
    assert!(!refreshed);
    assert_eq!(names(refresh.repos()), ["a"]);
    assert_eq!(refresh.due(), Some(Timestamp::from_unix(START + 300)));
    assert_eq!(
        *world.lines.borrow(),
        ["discovery failed reason=\"curl: exited with 6\""]
    );
}

#[test]
fn refresh_when_due_makes_the_merged_list_and_logs_the_change() {
    // Arrange
    let world = Rc::new(World::default());
    let mut refresh = refresh(&world, true);
    world.queue_discovery(Ok(vec![discovered("c"), discovered("b")]));
    world.queue_discovery(Ok(vec![discovered("c")]));

    // Act
    let first = refresh.refresh(Timestamp::from_unix(START));
    let after_first = names(refresh.repos()).join(" ");
    let second = refresh.refresh(Timestamp::from_unix(START + 3600));

    // Assert
    assert!(first && second);
    assert_eq!(after_first, "a b c");
    assert_eq!(names(refresh.repos()), ["a", "c"]);
    assert_eq!(
        *world.lines.borrow(),
        [
            "discovery repos=3 added=b,c removed=-",
            "discovery repos=2 added=- removed=b"
        ]
    );
    assert_eq!(refresh.due(), Some(Timestamp::from_unix(START + 7200)));
}

#[test]
fn refresh_without_a_discovery_is_never_due() {
    // Arrange
    let world = Rc::new(World::default());
    let mut refresh = refresh(&world, false);

    // Act
    let refreshed = refresh.refresh(Timestamp::from_unix(START));

    // Assert
    assert!(!refreshed);
    assert_eq!(names(refresh.repos()), ["a"]);
    assert_eq!(refresh.due(), None);
    assert_eq!(world.lines.borrow().len(), 0);
}
