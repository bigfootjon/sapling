#chg-compatible

  $ configure modern

  $ setconfig paths.default=test:e1 ui.traceback=1
  $ setconfig treemanifest.flatcompat=0
  $ setconfig infinitepush.httpbookmarks=1
  $ setconfig pull.httpbookmarks=1
  $ setconfig exchange.httpcommitlookup=1
  $ export LOG=exchange::httpcommitlookup=debug,pull::httpbookmarks=debug

Disable SSH:

  $ setconfig ui.ssh=false

Prepare Repo:

  $ newremoterepo
  $ setconfig paths.default=test:e1
  $ drawdag << 'EOS'
  > B C  # C/T/A=2
  > |/
  > A    # A/T/A=1
  > EOS

Push:

  $ hg push -r $C --to master --create
  pushing rev 178c10ffbc2f to destination test:e1 bookmark master
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\x17\x8c\x10\xff\xbc/\x92\xd5@|\x14G\x8a\xe9\xd9\xde\xa8\x1f#.', 'known': {'Ok': False}}
  searching for changes
  exporting bookmark master
  $ hg push -r $B --to remotebook --create
  pushing rev 99dac869f01e to destination test:e1 bookmark remotebook
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\x17\x8c\x10\xff\xbc/\x92\xd5@|\x14G\x8a\xe9\xd9\xde\xa8\x1f#.', 'known': {'Ok': True}}
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\x99\xda\xc8i\xf0\x1e\t\xfe=P\x1f\xa6E\xeaRJ\xf8\rI\x8f', 'known': {'Ok': False}}
  searching for changes
  exporting bookmark remotebook
  $ hg book --list-remote master
     master                    178c10ffbc2f92d5407c14478ae9d9dea81f232e

Pull:

  $ newremoterepo
  $ setconfig paths.default=test:e1
  $ hg debugchangelog --migrate lazy
  $ hg pull -B master
  pulling from test:e1
   DEBUG pull::httpbookmarks: edenapi fetched bookmarks: {'master': '178c10ffbc2f92d5407c14478ae9d9dea81f232e'}
  $ hg book --list-subscriptions
     remote/master             178c10ffbc2f

Pull with multiround sampling:
  $ drawdag << 'EOS'
  > F
  > |
  > E
  > EOS
C is known and E is unknown
  $ hg push -r $E --allow-anon
  pushing to test:e1
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\x17\x8c\x10\xff\xbc/\x92\xd5@|\x14G\x8a\xe9\xd9\xde\xa8\x1f#.', 'known': {'Ok': True}}
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\xe8\xe0\xa8\x1d\x95\x0f\xedk!}>\xc4U\xe6\x1a\xf1\xceN\xefH', 'known': {'Ok': False}}
  searching for changes
C and E are known and F is unknown
  $ hg pull -B remotebook
  pulling from test:e1
   DEBUG pull::httpbookmarks: edenapi fetched bookmarks: {'remotebook': '99dac869f01e09fe3d501fa645ea524af80d498f'}
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\x17\x8c\x10\xff\xbc/\x92\xd5@|\x14G\x8a\xe9\xd9\xde\xa8\x1f#.', 'known': {'Ok': True}}
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b"/'FJf\xa0\x1c\x1c\x14\xaa%yN\xf4\x10Q\x8d\xc0\x17\xaf", 'known': {'Ok': False}}
  searching for changes
   DEBUG exchange::httpcommitlookup: edenapi commitknown: {'hgid': b'\xe8\xe0\xa8\x1d\x95\x0f\xedk!}>\xc4U\xe6\x1a\xf1\xceN\xefH', 'known': {'Ok': True}}
  $ hg book --list-subscriptions
     remote/master             178c10ffbc2f
     remote/remotebook         99dac869f01e
