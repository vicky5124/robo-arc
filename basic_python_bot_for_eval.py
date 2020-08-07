import textwrap
import traceback

import lightbulb
import hikari
from hikari.models.intents import Intent

import toml

with open('config.toml', 'r') as f:
    raw_config = f.read()
    config = toml.loads(raw_config)

TOKEN = config["discord"]

PREFIX = ","

bot = lightbulb.Bot(
    token=TOKEN,
    intents=Intent.PRIVATE_MESSAGES | Intent.GUILD_MESSAGES,
    stateless=True,
    prefix="​1​7h23n9187q25bwe95rw6e e68 ​fs5e78f56s49d 8f7s5df4sd48f9sd34 ​​[broad-except] Catching too general exception Exception [W0703] s798df8sd6 4fs98d6f48​, yeah, nothing is triggering the command not found error anymore, unless someone looks ​at the source :P, in that case, ​hi nerd!​"
)

command = f"{PREFIX}eval"

@bot.listen()
async def ready(_event: hikari.ShardReadyEvent) -> None:
    await bot.fetch_owner_ids()

@bot.listen()
async def _eval(event: hikari.MessageCreateEvent) -> None:
    if event.message.author.id in bot.owner_ids and event.message.content.startswith(command):
        code = event.message.content[len(command):]
        code = code.strip("`py ")

        env = {
            "bot": bot,
            "client": bot,
            "msg": event.message,
            "message": event.message,
            "server_id": event.message.guild_id,
            "guild_id": event.message.guild_id,
            "channel_id": event.message.channel_id,
            "author": event.message.author,
        }
        env.update(globals())

        new_forced_async_code = f"async def code():\n{textwrap.indent(code, '    ')}"

        exec(new_forced_async_code, env) # shut up pylint with "[exec-used] Use of exec [W0122]"
        code = env["code"]

        try:
            result = await code()

            embed = hikari.Embed(
                title="Success!",
                description=f"Returned value: ```py\n{result}```",
                colour=(5, 255, 70)
            )
            await event.message.reply(embed=embed)
            await event.message.add_reaction("✅")
        except Exception as error: # shut up pylint with "[broad-except] Catching too general exception Exception [W0703]"
            embed = hikari.Embed(
                title="Failed to execute.",
                description=f"{error} ```py\n{traceback.format_exc()}```",
                colour=(255, 10, 40)
            )
            await event.message.reply(embed=embed)
            await event.message.add_reaction("❌")

bot.run()
