use db::{CharNote, CharPoint, CharValues, Character, NewCharacter};
use diesel::{self, prelude::*};
use key::DatabaseConnection;
use serenity::client::Context;

fn find_char_id(
    db: DatabaseConnection::Value,
    find_name: &str,
    find_game: Option<&str>,
    channel: Option<&str>,
) -> Option<i32> {
    use schema::characters::dsl::*;

    if game.is_some() {

        characters.filter(name.eq(name)


}

/*
!character create <NAME> in <GAME>
!character remove <NAME> in <GAME>
!character pin <NAME> [in <GAME>]
!character unpin <NAME> [in <GAME>]
!character rename <NAME> [in <GAME>] to <NEW_NAME>
!character move <NAME> [in <GAME>] to <NEW_GAME>

!character list [GAME]
!character view <NAME> [in GAME]

!character set <NAME> [in <GAME>] <+-=>00 <PP>
!character set <NAME> [in <GAME>] has <MAX> <PP>

!character set <NAME> [in <GAME>] <KEY> is <VALUE>
!character get <NAME> [in <GAME>] <KEY>
!character del <NAME> [in <GAME>] <KEY>

-- <NAME> [in <GAME>] falls back to the channel name for <GAME>
   if it cannot provide a unique result it returns an error.

*/
