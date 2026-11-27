use std::io;

use toolbox_exec::RouteProblem;
use toolbox_state::aur::{AurRoute, RouteKind};

use super::{RouteSource, probe_route};

struct Answer(Result<(), fn() -> RouteProblem>);

impl RouteSource for Answer {
    fn route(&self) -> Result<(), RouteProblem> {
        self.0.map_err(|problem| problem())
    }
}

fn refused_as(problem: fn() -> RouteProblem) -> RouteKind {
    match probe_route(&Answer(Err(problem))) {
        AurRoute::Refused { kind, detail: _ } => kind,
        AurRoute::Accepted => panic!("the route should be refused"),
    }
}

#[test]
fn an_accepted_route_is_the_accepted_part() {
    assert_eq!(probe_route(&Answer(Ok(()))), AurRoute::Accepted);
}

#[test]
fn each_problem_names_the_kind_a_text_tells_apart() {
    assert_eq!(
        refused_as(|| RouteProblem::NoAccount(1000)),
        RouteKind::NoAccount
    );
    assert_eq!(
        refused_as(|| RouteProblem::Home(missing())),
        RouteKind::Linked
    );
    assert_eq!(
        refused_as(|| RouteProblem::Folder {
            name: "garuda-toolbox".to_owned(),
            error: missing(),
        }),
        RouteKind::Linked
    );
    assert_eq!(
        refused_as(|| RouteProblem::Shared("/home/ada/.cache".to_owned())),
        RouteKind::Shared
    );
    assert_eq!(
        refused_as(|| RouteProblem::Foreign("/home/ada".to_owned())),
        RouteKind::Other
    );
}

#[test]
fn the_detail_is_the_problem_as_the_helper_words_it() {
    let refused = probe_route(&Answer(Err(|| RouteProblem::NoAccount(1000))));

    assert_eq!(
        refused,
        AurRoute::Refused {
            kind: RouteKind::NoAccount,
            detail: "user id 1000 has no account in /etc/passwd".to_owned(),
        }
    );
}

fn missing() -> io::Error {
    io::Error::from(io::ErrorKind::NotFound)
}
