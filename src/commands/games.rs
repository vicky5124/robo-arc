use crate::commands::moderation::parse_member;
use crate::utils::checks::*;

use std::fs;
use std::fmt::Display;

use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

use std::time::Duration;

use serenity::{
    prelude::Context,
    model::misc::Mentionable,
    model::channel::{
        Message,
        ReactionType,
    },
    model::user::User,
    model::id::UserId,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
};

/// Play some Higher or Lower.
/// You don't get anything in reward for playing this, gambling is bad.
#[command]
#[aliases(hol, higherorlower)]
async fn higher_or_lower(ctx: &Context, msg: &Message) -> CommandResult {
    let cards = fs::read_dir("poker_cards")?.map(|i| {
        let f = i.unwrap();
        f.file_name().into_string().unwrap()
    }).collect::<Vec<String>>();

    let mut rng = StdRng::from_entropy();
    let choice = &cards.choose(&mut rng).unwrap();

    let mut message = msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Higher or Lower");
            e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
        })
    }).await?;

    let up = ReactionType::Unicode("⬆️".to_string());
    let down = ReactionType::Unicode("⬇️".to_string());

    message.react(ctx, up).await?;
    message.react(ctx, down).await?;

    let mut iteration = 1u8;
    let mut current_value = choice.split('.').next().unwrap()[1..].parse::<u8>()?;

    loop {
        if let Some(reaction) = message.await_reaction(ctx).author_id(msg.author.id).timeout(Duration::from_secs(120)).await {
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
                            message.edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title(format!("{} lost.", msg.author.name));
                                    e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                                })
                            }).await?;

                            break
                        }
                    } else {
                        if new_value > current_value {
                            message.edit(ctx, |m| {
                                m.embed(|e| {
                                    e.title(format!("{} lost.", msg.author.name));
                                    e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                                })
                            }).await?;

                            break 
                        }
                    }

                    current_value = new_value;

                    iteration += 1;

                    if iteration > 3 {
                        message.edit(ctx, |m| {
                            m.embed(|e| {
                                e.title(format!("{} won!", msg.author.name));
                                e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                            })
                        }).await?;

                        break 
                    } else {
                        message.edit(ctx, |m| {
                            m.embed(|e| {
                                e.title("Higher or Lower");
                                e.image(format!("https://5124.mywire.org/HDD/poker_cards/{}", choice))
                            })
                        }).await?;
                    }
                },
                _ => (),
            }
        } else {
            message.edit(ctx, |m| {
                m.embed(|e| {
                    e.title("Timeout!")
                })
            }).await?;
            break
        }
    }

    let _ = message.delete_reactions(ctx).await;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
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
    fn default() -> Self { Pieces::Cross }
}

impl Display for Pieces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Cross => "X",
            Self::Circle => "O",
        })
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = format!("{} | A | B | C\n", self.current_piece);
        board += "--------------";

        let mut x = 0;
        for (index, i) in self.table.iter().enumerate() {
            if index % 3 == 0 {
                x+=1;
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
    fn place_piece(&mut self, piece: Piece) {
        let x = piece.pos_x * 3;
        let y = piece.pos_y % 3;
        self.table[x+y] = piece;
    }
    fn swap_current_piece(&mut self) {
        self.current_piece = match self.current_piece {
            Pieces::Cross => Pieces::Circle,
            Pieces::Circle => Pieces::Cross,
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
#[checks("bot_has_manage_messages")]
//#[min_args(1)]
async fn tic_tac_toe(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let mut players = [
        Player(msg.author.id, Pieces::Cross),
        Player(msg.author.id, Pieces::Circle),
    ].repeat(5);

    players.pop();

    let mut board = Board::default();
    let mut m = msg.channel_id.say(ctx, format!(">>> ```{}```", &board)).await?;

    for i in players {
        let piece = Piece {
            pos_x: 1,
            pos_y: 1,
            typ: Some(Pieces::Cross),
        };

        board.place_piece(piece);
        board.swap_current_piece();

        m.edit(ctx, |m| m.content(format!("{}\n>>> ```{}```", i.0.mention(), &board))).await?;
    }

    Ok(())
}


