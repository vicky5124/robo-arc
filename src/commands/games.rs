use crate::commands::moderation::parse_member;

use std::fmt::Display;
use std::fs;
use std::time::Duration;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::{Message, ReactionType},
    model::id::UserId,
    model::misc::Mentionable,
    prelude::Context,
};

/// Play some Higher or Lower.
/// You don't get anything in reward for playing this, gambling is bad.
#[command]
#[aliases(hol, higherorlower)]
async fn higher_or_lower(ctx: &Context, msg: &Message) -> CommandResult {
    let cards = fs::read_dir("poker_cards")?
        .map(|i| {
            let f = i.unwrap();
            f.file_name().into_string().unwrap()
        })
        .collect::<Vec<String>>();

    let mut rng = StdRng::from_entropy();
    let choice = &cards.choose(&mut rng).unwrap();

    let mut message = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Higher or Lower");
                e.image(format!(
                    "https://5124.mywire.org/HDD/poker_cards/{}",
                    choice
                ))
            })
        })
        .await?;

    let up = ReactionType::Unicode("⬆️".to_string());
    let down = ReactionType::Unicode("⬇️".to_string());

    message.react(ctx, up).await?;
    message.react(ctx, down).await?;

    let mut iteration = 1u8;
    let mut current_value = choice.split('.').next().unwrap()[1..].parse::<u8>()?;

    loop {
        if let Some(reaction) = message
            .await_reaction(ctx)
            .author_id(msg.author.id)
            .timeout(Duration::from_secs(120))
            .await
        {
            let emoji = &reaction.as_inner_ref().emoji;
            let emoji_data = emoji.as_data();
            let emoji_str = emoji_data.as_str();

            match emoji_str {
                "⬆️" | "⬇️" => {
                    let higher = emoji_str == "⬆️";

                    let choice = &cards.choose(&mut rng).unwrap();
                    let new_value = choice.split('.').next().unwrap()[1..].parse::<u8>()?;

                    if higher {
                        if new_value < current_value {
                            message
                                .edit(ctx, |m| {
                                    m.embed(|e| {
                                        e.title(format!("{} lost.", msg.author.name));
                                        e.image(format!(
                                            "https://5124.mywire.org/HDD/poker_cards/{}",
                                            choice
                                        ))
                                    })
                                })
                                .await?;

                            break;
                        }
                    } else {
                        if new_value > current_value {
                            message
                                .edit(ctx, |m| {
                                    m.embed(|e| {
                                        e.title(format!("{} lost.", msg.author.name));
                                        e.image(format!(
                                            "https://5124.mywire.org/HDD/poker_cards/{}",
                                            choice
                                        ))
                                    })
                                })
                                .await?;

                            break;
                        }
                    }

                    current_value = new_value;

                    iteration += 1;

                    if iteration > 3 {
                        message
                            .edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title(format!("{} won!", msg.author.name));
                                    e.image(format!(
                                        "https://5124.mywire.org/HDD/poker_cards/{}",
                                        choice
                                    ))
                                })
                            })
                            .await?;

                        break;
                    } else {
                        message
                            .edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title("Higher or Lower");
                                    e.image(format!(
                                        "https://5124.mywire.org/HDD/poker_cards/{}",
                                        choice
                                    ))
                                })
                            })
                            .await?;
                    }
                }
                _ => (),
            }
        } else {
            message
                .edit(ctx, |m| m.embed(|e| e.title("Timeout!")))
                .await?;
            break;
        }
    }

    let _ = message.delete_reactions(ctx).await;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pieces {
    Cross,
    Circle,
}

#[derive(Debug, Clone, Copy)]
struct Player(UserId, Pieces);

#[derive(Default, Debug)]
struct Piece {
    pos_x: usize,
    pos_y: usize,
    typ: Option<Pieces>,
}

#[derive(Default, Debug)]
struct Board {
    table: [Piece; 9],
    current_piece: Pieces,
    win_condition: Option<Pieces>,
}

impl Default for Pieces {
    fn default() -> Self {
        Pieces::Cross
    }
}

impl Display for Pieces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cross => "X",
                Self::Circle => "O",
            }
        )
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = format!("{} | A | B | C\n", self.current_piece);
        board += "--------------";

        let mut x = 0;
        for (index, i) in self.table.iter().enumerate() {
            if index % 3 == 0 {
                x += 1;
                board += &format!("\n{} ", x);
            }

            board += &format!("| {} ", {
                if let Some(piece) = &i.typ {
                    piece.to_string()
                } else {
                    " ".to_string()
                }
            });
        }

        write!(f, "{}", board)
    }
}

impl Board {
    fn place_piece(&mut self, piece: Piece) -> Result<(), ()> {
        let x = piece.pos_x * 3;
        let y = piece.pos_y % 3;
        if self.table[x + y].typ.is_none() {
            self.table[x + y] = piece;
            Ok(())
        } else {
            Err(())
        }
    }
    fn swap_current_piece(&mut self) {
        self.current_piece = match self.current_piece {
            Pieces::Cross => Pieces::Circle,
            Pieces::Circle => Pieces::Cross,
        }
    }

    fn check_win_condition(&mut self) {
        let win_conditions = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [6, 4, 2],
        ];

        for i in &win_conditions {
            if self.table[i[0]].typ == Some(Pieces::Cross)
                && self.table[i[1]].typ == Some(Pieces::Cross)
                && self.table[i[2]].typ == Some(Pieces::Cross)
            {
                self.win_condition = Some(Pieces::Cross);
            }
            if self.table[i[0]].typ == Some(Pieces::Circle)
                && self.table[i[1]].typ == Some(Pieces::Circle)
                && self.table[i[2]].typ == Some(Pieces::Circle)
            {
                self.win_condition = Some(Pieces::Circle);
            }
        }
    }
}

/// 2 player game where you must compete with the other player to be the first to obtain 3 of your pieces in line.
///
/// When it's your turn, react with a number and a letter, corresponding to the position of the board.
/// If the place is taken, you will need to repick the position.
///
/// Is there an AI to play by myself? No, you have to play with another player.
///
/// Usage:
/// `ttt @timmy`
#[command]
#[aliases(ttt, tictactoe)]
#[min_args(1)]
async fn tic_tac_toe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let other_player = parse_member(ctx, &msg, args.single_quoted::<String>()?).await?;

    let mut confirmation = msg
        .channel_id
        .say(
            ctx,
            format!(
                "{}: Do you accept this TicTacToe match?",
                other_player.mention()
            ),
        )
        .await?;
    confirmation.react(ctx, '✅').await?;
    confirmation.react(ctx, '❌').await?;

    loop {
        if let Some(reaction) = other_player
            .user
            .await_reaction(ctx)
            .timeout(Duration::from_secs(120))
            .await
        {
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
                "✅" => {
                    confirmation.delete(ctx).await?;
                    break;
                }
                "❌" => {
                    confirmation
                        .edit(ctx, |m| {
                            m.content(format!(
                                "{}: {} didn't accept the match.",
                                msg.author.mention(),
                                other_player.mention()
                            ))
                        })
                        .await?;
                    return Ok(());
                }
                _ => (),
            }
        } else {
            confirmation
                .edit(ctx, |m| {
                    m.content(format!(
                        "{}: {} took to long to respond.",
                        msg.author.mention(),
                        other_player.mention()
                    ))
                })
                .await?;
            return Ok(());
        }
    }

    let mut players = [
        Player(msg.author.id, Pieces::Cross),
        Player(other_player.user.id, Pieces::Circle),
    ]
    .repeat(5);

    if msg.timestamp.timestamp() % 2 == 0 {
        players.reverse();
    }
    players.pop();

    let mut board = Board::default();
    board.current_piece = players[0].1;
    let mut m = msg
        .channel_id
        .say(ctx, format!(">>> ```{}```", &board))
        .await?;

    for i in 1..4u8 {
        let num = ReactionType::Unicode(String::from(format!("{}\u{fe0f}\u{20e3}", i)));
        m.react(ctx, num).await?;
    }

    let _a = ReactionType::Unicode(String::from("\u{01f1e6}"));
    let _b = ReactionType::Unicode(String::from("\u{01f1e7}"));
    let _c = ReactionType::Unicode(String::from("\u{01f1e8}"));

    m.react(ctx, _a).await?;
    m.react(ctx, _b).await?;
    m.react(ctx, _c).await?;

    for i in &players {
        m.edit(ctx, |m| {
            m.content(format!("{}\n>>> ```{}```", i.0.mention(), &board))
        })
        .await?;

        'outer: loop {
            let mut x: Option<usize> = None;
            let mut y: Option<usize> = None;
            loop {
                if x.is_none() || y.is_none() {
                    if let Some(reaction) =
                        i.0.to_user(ctx)
                            .await?
                            .await_reaction(ctx)
                            .timeout(Duration::from_secs(120))
                            .await
                    {
                        let _ = reaction.as_inner_ref().delete(ctx).await;
                        let emoji = &reaction.as_inner_ref().emoji;

                        match emoji.as_data().as_str() {
                            "1\u{fe0f}\u{20e3}" => x = Some(0),
                            "2\u{fe0f}\u{20e3}" => x = Some(1),
                            "3\u{fe0f}\u{20e3}" => x = Some(2),
                            "\u{01f1e6}" => y = Some(0),
                            "\u{01f1e7}" => y = Some(1),
                            "\u{01f1e8}" => y = Some(2),
                            _ => (),
                        }
                    } else {
                        m.edit(ctx, |m| m.content(format!("{}: Timeout", i.0.mention())))
                            .await?;
                        let _ = m.delete_reactions(ctx).await;
                        return Ok(());
                    }
                } else {
                    if !x.is_none() && !y.is_none() {
                        let piece = Piece {
                            pos_x: x.unwrap(),
                            pos_y: y.unwrap(),
                            typ: Some(i.1),
                        };
                        if let Err(_) = board.place_piece(piece) {
                            x = None;
                            y = None;
                        } else {
                            break 'outer;
                        }
                    }
                }
            }
        }
        board.check_win_condition();

        if let Some(_) = board.win_condition {
            m.edit(ctx, |m| {
                m.content(format!("{} WON!\n>>> ```{}```", i.0.mention(), &board))
            })
            .await?;
            let _ = m.delete_reactions(ctx).await;
            return Ok(());
        }
        board.swap_current_piece();
    }
    m.edit(ctx, |m| {
        m.content(format!(
            "{} and {} tied.\n>>> ```{}```",
            players[0].0.mention(),
            players[1].0.mention(),
            &board
        ))
    })
    .await?;
    let _ = m.delete_reactions(ctx).await;

    Ok(())
}
