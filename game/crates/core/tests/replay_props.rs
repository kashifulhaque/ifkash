use lf_core::input::{ActionFrame, Command};
use lf_core::replay::{Replay, ReplayHeader, compress, expand};
use proptest::prelude::*;

fn arb_frame() -> impl Strategy<Value = ActionFrame> {
    // Few distinct values so runs actually occur.
    (-1i8..=1, -1i8..=1, 0u16..3).prop_map(|(move_x, move_z, buttons)| ActionFrame {
        move_x,
        move_z,
        buttons,
    })
}

fn arb_command() -> impl Strategy<Value = Command> {
    prop_oneof![
        Just(Command::Rest),
        any::<u8>().prop_map(|attribute| Command::LevelUp { attribute }),
        (any::<u8>(), any::<u32>()).prop_map(|(slot, item)| Command::Equip { slot, item }),
        any::<u8>().prop_map(|slot| Command::Unequip { slot }),
        any::<u8>().prop_map(|slot| Command::UseItem { slot }),
        any::<u16>().prop_map(Command::Gesture),
    ]
}

proptest! {
    #[test]
    fn any_replay_round_trips(
        runs in proptest::collection::vec((any::<(i8, i8, u16)>(), 0u32..5), 0..40),
        commands in proptest::collection::vec((any::<u64>(), arb_command()), 0..20),
        hashes in proptest::collection::vec((any::<u64>(), any::<u64>()), 0..10),
        seed in any::<u64>(),
        content_hash in any::<u64>(),
        scenario in ".{0,20}",
        start_tick in any::<u64>(),
    ) {
        let r = Replay {
            header: ReplayHeader { format_version: 1, content_hash, seed, scenario, start_tick },
            frames: runs.into_iter().map(|((x, z, b), n)| (ActionFrame { move_x: x, move_z: z, buttons: b }, n)).collect(),
            commands,
            hashes,
        };
        prop_assert_eq!(Replay::decode(&r.encode()).unwrap(), r);
    }

    #[test]
    fn recorded_replay_round_trips(
        frames in proptest::collection::vec(arb_frame(), 0..200),
        commands in proptest::collection::vec((0u64..200, arb_command()), 0..10),
        seed in any::<u64>(),
    ) {
        let r = Replay::record(seed, "m0_flat", 1, &frames, commands);
        let back = Replay::decode(&r.encode()).unwrap();
        prop_assert_eq!(&back, &r);
        prop_assert_eq!(expand(&back.frames), frames);
    }

    #[test]
    fn rle_recompress_is_identity(frames in proptest::collection::vec(arb_frame(), 0..300)) {
        let runs = compress(&frames);
        prop_assert!(runs.windows(2).all(|w| w[0].0 != w[1].0));
        prop_assert!(runs.iter().all(|&(_, n)| n >= 1));
        prop_assert_eq!(compress(&expand(&runs)), runs);
    }

    #[test]
    fn frame_bytes_round_trip(x in any::<i8>(), z in any::<i8>(), b in any::<u16>()) {
        let f = ActionFrame { move_x: x, move_z: z, buttons: b };
        prop_assert_eq!(ActionFrame::from_bytes(f.to_bytes()), f);
    }
}
