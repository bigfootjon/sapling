# Copyright 2016-present Facebook. All Rights Reserved.
#
# fastannotate: faster annotate implementation using linelog
#
# This software may be used and distributed according to the terms of the
# GNU General Public License version 2 or any later version.


"""yet another annotate implementation that might be faster

The fastannotate extension provides a 'fastannotate' command that makes
use of the linelog data structure as a cache layer and is expected to
be faster than the vanilla 'annotate' if the cache is present.

::

    [fastannotate]
    # specify the main branch head. the internal linelog will only contain
    # the linear (ignoring p2) "mainbranch". since linelog cannot move
    # backwards without a rebuild, this should be something that always moves
    # forward, usually it is "master" or "@".
    mainbranch = master

    # add a "fastannotate" command, and replace the default "annotate" command
    commands = fastannotate, annotate

    # default format when no format flags are used (default: number)
    defaultformat = changeset, user, date

    # replace hgweb's annotate implementation (default: False)
    # note: mainbranch should be set to a forward-only name, otherwise the
    # linelog cache may be rebuilt frequently, which leads to errors and
    # poor performance
    hgweb = True

    # serve the annotate cache via wire protocol (default: False)
    # tip: the .hg/fastannotate directory is portable - can be rsynced
    server = True

    # update local annotate cache from remote on demand
    # (default: True for remotefilelog repo, False otherwise)
    client = True

    # path to use when connecting to the remote server (default: default)
    remotepath = default

    # use flock instead of the file existence lock
    # flock may not work well on some network filesystems, but they avoid
    # creating and deleting files frequently, which is faster when updating
    # the annotate cache in batch. if you have issues with this option, set it
    # to False. (default: True if flock is supported, False otherwise)
    useflock = True

    # use unfiltered repo for better performance
    unfilteredrepo = True

    # sacrifice correctness in some corner cases for performance. it does not
    # affect the correctness of the annotate cache being built. the option
    # is experimental and may disappear in the future (default: False)
    perfhack = True
"""

from __future__ import absolute_import

from mercurial.i18n import _
from mercurial import (
    error as hgerror,
    util,
)

from . import (
    commands,
    context,
    protocol,
)

testedwith = 'internal'

cmdtable = commands.cmdtable

def _flockavailable():
    try:
        import fcntl
        fcntl.flock
    except StandardError:
        return False
    else:
        return True

def uisetup(ui):
    cmdnames = ui.configlist('fastannotate', 'commands', ['fastannotate'])
    for name in set(cmdnames):
        if name == 'fastannotate':
            commands.registercommand()
        elif name == 'annotate':
            commands.replacedefault()
        else:
            raise hgerror.Abort(_('%s: invalid fastannotate.commands option')
                                % name)
    if ui.configbool('fastannotate', 'hgweb'):
        # local import to avoid overhead of loading hgweb for non-hgweb usages
        from . import hgwebsupport
        hgwebsupport.replacehgwebannotate()

    if ui.configbool('fastannotate', 'server'):
        protocol.serveruisetup(ui)

    if ui.configbool('fastannotate', 'useflock', _flockavailable()):
        context.pathhelper.lock = context.pathhelper._lockflock

def reposetup(ui, repo):
    client = ui.configbool('fastannotate', 'client', default=None)
    if client is None:
        if util.safehasattr(repo, 'requirements'):
            client = 'remotefilelog' in repo.requirements
    if client:
        protocol.clientreposetup(ui, repo)
