import io
import sys
import textwrap
import traceback
from contextlib import redirect_stdout

import lightbulb
import hikari
from hikari.models.intents import Intents

import toml

with open('config.toml', 'r') as f:
    raw_config = f.read()
    config = toml.loads(raw_config)

TOKEN = config["discord"]

PREFIX = "."

bot = lightbulb.Bot(
    token=TOKEN,
    intents=Intents.PRIVATE_MESSAGES | Intents.GUILD_MESSAGES,
    stateless=True,
    prefix="​1​7h23n9187q25bwe95rw6e e68 ​fs5e78f56s49d 8f7s5df4sd48f9sd34 ​​[broad-except] Catching too general exception Exception [W0703] s798df8sd6 4fs98d6f48​, yeah, nothing is triggering the command not found error anymore, unless someone looks ​at the source :P, in that case, ​hi nerd!​"
)

command = f"{PREFIX}eval"

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

@bot.listen()
async def ready(_event: hikari.ShardReadyEvent) -> None:
    await bot.fetch_owner_ids()

@bot.listen()
async def _eval(event: hikari.MessageCreateEvent) -> None:
    if event.message.author.id in bot.owner_ids and event.message.content.startswith(command):
        code = event.message.content[len(command) + 1:]

        if code.startswith('```') and code.endswith('```'):
            code = '\n'.join(code.split('\n')[1:-1])
        else:
            code = code.strip('` \n')

        env = {
            "bot": bot,
            "client": bot,
            "msg": event.message,
            "message": event.message,
            "server_id": event.message.guild_id,
            "guild_id": event.message.guild_id,
            "channel_id": event.message.channel_id,
            "author": event.message.author,
            "eprint": eprint,
        }
        env.update(globals())
        stdout = io.StringIO()

        new_forced_async_code = f"async def code():\n{textwrap.indent(code, '    ')}"

        try:
            exec(new_forced_async_code, env) # shut up pylint with "[exec-used] Use of exec [W0122]"
        except Exception as error: # shut up pylint with "[broad-except] Catching too general exception Exception [W0703]"
            embed = hikari.Embed(
                title="Failed to execute.",
                description=f"{error} ```py\n{traceback.format_exc()}\n```\n```py\n{error.__class__.__name__}\n```",
                colour=(255, 10, 40)
            )
            await event.message.reply(embed=embed)
            await event.message.add_reaction("❌")
            return

        code = env["code"]

        try:
            with redirect_stdout(stdout):
                result = await code()
        except Exception as error: # shut up pylint with "[broad-except] Catching too general exception Exception [W0703]"
            value = stdout.getvalue()
            embed = hikari.Embed(
                title="Failed to execute.",
                description=f"{error} ```py\n{traceback.format_exc()}\n```\n```py\n{value}\n```",
                colour=(255, 10, 40)
            )
            await event.message.reply(embed=embed)
            await event.message.add_reaction("❌")
            return

        value = stdout.getvalue()
        embed = hikari.Embed(
            title="Success!",
            description=f"Returned value: ```py\n{result}\n```\nStandard Output: ```py\n{value}\n```",
            colour=(5, 255, 70)
        )
        await event.message.reply(embed=embed)
        await event.message.add_reaction("✅")




bot.run()
