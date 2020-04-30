import textwrap
import traceback
import toml
import discord
from discord.ext import commands

with open('config.toml', 'r') as f:
    raw_config = f.read()
    config = toml.loads(raw_config)

TOKEN = config["discord"]


BOT = commands.Bot(command_prefix=commands.when_mentioned_or("."))
BOT.remove_command('help')

# Intents seem to be broken?
#print(BOT.intents)
#intents = discord.Intents.none()

@BOT.event
async def on_ready():
    print(f"Python BOT is ready.")

@BOT.event
async def on_message(msg):
    app_info = await BOT.application_info()
    if msg.author != app_info.owner:
        return

    await BOT.process_commands(msg)

@BOT.command(name='eval')
@commands.is_owner()
async def _eval(ctx, *, code):
    if "import os" in code or "import sys" in code:
        return

    code = code.strip('` ')

    env = {
        'bot': BOT,
        'client': BOT,
        'ctx': ctx,
        'message': ctx.message,
        'server': ctx.message.guild,
        'guild': ctx.message.guild,
        'channel': ctx.message.channel,
        'author': ctx.message.author,
    }
    env.update(globals())

    new_forced_async_code = f'async def code():\n{textwrap.indent(code, "    ")}'

    exec(new_forced_async_code, env)
    code = env['code']

    try:
        await code()
    except:
        await ctx.send(f'```{traceback.format_exc()}```')
        await ctx.message.add_reaction('❌')

@BOT.event
async def on_error(_):
    return

@BOT.event
async def on_command_error(_1, _2):
    return

BOT.run(TOKEN)
