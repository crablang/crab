#!/usr/bin/env python
# -*- coding: utf-8 -*-

# This script computes the new "current" toolstate for the toolstate repo (not to be
# confused with publishing the test results, which happens in `src/bootstrap/toolstate.rs`).
# It gets called from `src/ci/publish_toolstate.sh` when a new commit lands on `master`
# (i.e., after it passed all checks on `auto`).

from __future__ import print_function

import sys
import re
import os
import json
import datetime
import collections
import textwrap
try:
    import urllib2
    from urllib2 import HTTPError
except ImportError:
    import urllib.request as urllib2
    from urllib.error import HTTPError
try:
    import typing
except ImportError:
    pass

# List of people to ping when the status of a tool or a book changed.
# These should be collaborators of the crablang/crablang repository (with at least
# read privileges on it). CI will fail otherwise.
MAINTAINERS = {
    'book': {'carols10cents'},
    'nomicon': {'frewsxcv', 'Gankra', 'JohnTitor'},
    'reference': {'Havvy', 'matthewjasper', 'ehuss'},
    'crablang-by-example': {'marioidival'},
    'embedded-book': {'adamgreig', 'andre-richter', 'jamesmunns', 'therealprof'},
    'edition-guide': {'ehuss'},
    'crablangc-dev-guide': {'spastorino', 'amanjeev', 'JohnTitor'},
}

LABELS = {
    'book': ['C-bug'],
    'nomicon': ['C-bug'],
    'reference': ['C-bug'],
    'crablang-by-example': ['C-bug'],
    'embedded-book': ['C-bug'],
    'edition-guide': ['C-bug'],
    'crablangc-dev-guide': ['C-bug'],
}

REPOS = {
    'book': 'https://github.com/crablang/book',
    'nomicon': 'https://github.com/crablang/nomicon',
    'reference': 'https://github.com/crablang/reference',
    'crablang-by-example': 'https://github.com/crablang/crablang-by-example',
    'embedded-book': 'https://github.com/crablang-embedded/book',
    'edition-guide': 'https://github.com/crablang/edition-guide',
    'crablangc-dev-guide': 'https://github.com/crablang/crablangc-dev-guide',
}

def load_json_from_response(resp):
    # type: (typing.Any) -> typing.Any
    content = resp.read()
    if isinstance(content, bytes):
        content_str = content.decode('utf-8')
    else:
        print("Refusing to decode " + str(type(content)) + " to str")
    return json.loads(content_str)


def read_current_status(current_commit, path):
    # type: (str, str) -> typing.Mapping[str, typing.Any]
    '''Reads build status of `current_commit` from content of `history/*.tsv`
    '''
    with open(path, 'r') as f:
        for line in f:
            (commit, status) = line.split('\t', 1)
            if commit == current_commit:
                return json.loads(status)
    return {}


def gh_url():
    # type: () -> str
    return os.environ['TOOLSTATE_ISSUES_API_URL']


def maybe_delink(message):
    # type: (str) -> str
    if os.environ.get('TOOLSTATE_SKIP_MENTIONS') is not None:
        return message.replace("@", "")
    return message


def issue(
    tool,
    status,
    assignees,
    relevant_pr_number,
    relevant_pr_user,
    labels,
    github_token,
):
    # type: (str, str, typing.Iterable[str], str, str, typing.List[str], str) -> None
    '''Open an issue about the toolstate failure.'''
    if status == 'test-fail':
        status_description = 'has failing tests'
    else:
        status_description = 'no longer builds'
    request = json.dumps({
        'body': maybe_delink(textwrap.dedent('''\
        Hello, this is your friendly neighborhood mergebot.
        After merging PR {}, I observed that the tool {} {}.
        A follow-up PR to the repository {} is needed to fix the fallout.

        cc @{}, do you think you would have time to do the follow-up work?
        If so, that would be great!
        ''').format(
            relevant_pr_number, tool, status_description,
            REPOS.get(tool), relevant_pr_user
        )),
        'title': '`{}` no longer builds after {}'.format(tool, relevant_pr_number),
        'assignees': list(assignees),
        'labels': labels,
    })
    print("Creating issue:\n{}".format(request))
    response = urllib2.urlopen(urllib2.Request(
        gh_url(),
        request.encode(),
        {
            'Authorization': 'token ' + github_token,
            'Content-Type': 'application/json',
        }
    ))
    response.read()


def update_latest(
    current_commit,
    relevant_pr_number,
    relevant_pr_url,
    relevant_pr_user,
    pr_reviewer,
    current_datetime,
    github_token,
):
    # type: (str, str, str, str, str, str, str) -> str
    '''Updates `_data/latest.json` to match build result of the given commit.
    '''
    with open('_data/latest.json', 'r+') as f:
        latest = json.load(f, object_pairs_hook=collections.OrderedDict)

        current_status = {
            os: read_current_status(current_commit, 'history/' + os + '.tsv')
            for os in ['windows', 'linux']
        }

        slug = 'crablang/crablang'
        message = textwrap.dedent('''\
            📣 Toolstate changed by {}!

            Tested on commit {}@{}.
            Direct link to PR: <{}>

        ''').format(relevant_pr_number, slug, current_commit, relevant_pr_url)
        anything_changed = False
        for status in latest:
            tool = status['tool']
            changed = False
            create_issue_for_status = None  # set to the status that caused the issue

            for os, s in current_status.items():
                old = status[os]
                new = s.get(tool, old)
                status[os] = new
                maintainers = ' '.join('@'+name for name in MAINTAINERS.get(tool, ()))
                # comparing the strings, but they are ordered appropriately:
                # "test-pass" > "test-fail" > "build-fail"
                if new > old:
                    # things got fixed or at least the status quo improved
                    changed = True
                    message += '🎉 {} on {}: {} → {} (cc {}).\n' \
                        .format(tool, os, old, new, maintainers)
                elif new < old:
                    # tests or builds are failing and were not failing before
                    changed = True
                    title = '💔 {} on {}: {} → {}' \
                        .format(tool, os, old, new)
                    message += '{} (cc {}).\n' \
                        .format(title, maintainers)
                    # See if we need to create an issue.
                    # Create issue if things no longer build.
                    # (No issue for mere test failures to avoid spurious issues.)
                    if new == 'build-fail':
                        create_issue_for_status = new

            if create_issue_for_status is not None:
                try:
                    issue(
                        tool, create_issue_for_status, MAINTAINERS.get(tool, ()),
                        relevant_pr_number, relevant_pr_user, LABELS.get(tool, []),
                        github_token,
                    )
                except HTTPError as e:
                    # network errors will simply end up not creating an issue, but that's better
                    # than failing the entire build job
                    print("HTTPError when creating issue for status regression: {0}\n{1!r}"
                          .format(e, e.read()))
                except IOError as e:
                    print("I/O error when creating issue for status regression: {0}".format(e))
                except:
                    print("Unexpected error when creating issue for status regression: {0}"
                          .format(sys.exc_info()[0]))
                    raise

            if changed:
                status['commit'] = current_commit
                status['datetime'] = current_datetime
                anything_changed = True

        if not anything_changed:
            return ''

        f.seek(0)
        f.truncate(0)
        json.dump(latest, f, indent=4, separators=(',', ': '))
        return message


# Warning: Do not try to add a function containing the body of this try block.
# There are variables declared within that are implicitly global; it is unknown
# which ones precisely but at least this is true for `github_token`.
try:
    if __name__ != '__main__':
        exit(0)

    cur_commit = sys.argv[1]
    cur_datetime = datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ')
    cur_commit_msg = sys.argv[2]
    save_message_to_path = sys.argv[3]
    github_token = sys.argv[4]

    # assume that PR authors are also owners of the repo where the branch lives
    relevant_pr_match = re.search(
        r'Auto merge of #([0-9]+) - ([^:]+):[^,]+, r=(\S+)',
        cur_commit_msg,
    )
    if relevant_pr_match:
        number = relevant_pr_match.group(1)
        relevant_pr_user = relevant_pr_match.group(2)
        relevant_pr_number = 'crablang/crablang#' + number
        relevant_pr_url = 'https://github.com/crablang/crablang/pull/' + number
        pr_reviewer = relevant_pr_match.group(3)
    else:
        number = '-1'
        relevant_pr_user = 'ghost'
        relevant_pr_number = '<unknown PR>'
        relevant_pr_url = '<unknown>'
        pr_reviewer = 'ghost'

    message = update_latest(
        cur_commit,
        relevant_pr_number,
        relevant_pr_url,
        relevant_pr_user,
        pr_reviewer,
        cur_datetime,
        github_token,
    )
    if not message:
        print('<Nothing changed>')
        sys.exit(0)

    print(message)

    if not github_token:
        print('Dry run only, not committing anything')
        sys.exit(0)

    with open(save_message_to_path, 'w') as f:
        f.write(message)

    # Write the toolstate comment on the PR as well.
    issue_url = gh_url() + '/{}/comments'.format(number)
    response = urllib2.urlopen(urllib2.Request(
        issue_url,
        json.dumps({'body': maybe_delink(message)}).encode(),
        {
            'Authorization': 'token ' + github_token,
            'Content-Type': 'application/json',
        }
    ))
    response.read()
except HTTPError as e:
    print("HTTPError: %s\n%r" % (e, e.read()))
    raise
