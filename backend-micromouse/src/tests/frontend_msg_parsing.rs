use crate::{
    comm::website::FrontendResponse,
    strategy::{
        dyn_strategy_tree::{DynStrategyConfig, StrategyChangeCommand},
        strategies::{
            breadth_first::BreadthFirstConfig, dbg_known_path::DbgKnownPathConfig,
            depth_first::DepthFirstConfig,
        },
        strategy::GoalPosition,
    },
    transform::position::Position,
};

#[test]
fn test_parse_frontend_msg() {
    use crate::comm::website::FrontendConnectionManagerInternal as FM;

    const N: usize = 10;

    assert!(FM::<N>::parse_msg("X").is_err());
    assert_eq!(
        FM::<N>::parse_msg(r#""Pause""#).unwrap(),
        FrontendResponse::Pause
    );
    assert_eq!(
        FM::<N>::parse_msg(r#""Continue""#).unwrap(),
        FrontendResponse::Continue
    );
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": false
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: false,
            set_strategy: None,
            set_goal: None
        })
    );
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: None,
            set_goal: None
        })
    );
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true,
            "set_position": null
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: None,
            set_goal: None
        })
    );
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true,
            "set_position": null,
            "set_strategy": null,
            "set_goal": null
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: None,
            set_goal: None
        })
    );
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true,
            "set_goal": {"x": 1, "y": 2}
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: None,
            set_goal: Some(GoalPosition(Position { x: 1, y: 2 }))
        })
    );
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true,
            "set_strategy": {
                "DepthFirst" : {
                    "forward_first": true
                }
            }
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: Some(DynStrategyConfig::DepthFirst(DepthFirstConfig {
                forward_first: true
            })),
            set_goal: None
        })
    );

    // Example with no config-values
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true,
            "set_strategy": {
                "DbgKnownPath": null
            }
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: Some(DynStrategyConfig::DbgKnownPath(DbgKnownPathConfig)),
            set_goal: None
        })
    );

    // Example with no config-values
    assert_eq!(
        FM::<N>::parse_msg(
            r#"
    {
        "StrategyChange" : {
            "reset_map": true,
            "set_strategy": {
                "BreadthFirst": null
            }
        }
    }
    "#
        )
        .unwrap(),
        FrontendResponse::StrategyChange(StrategyChangeCommand {
            set_postion: None,
            reset_map: true,
            set_strategy: Some(DynStrategyConfig::BreadthFirst(BreadthFirstConfig)),
            set_goal: None
        })
    );
}
