/*
 * parse/rule/impls/underline.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use super::prelude::*;

pub const RULE_UNDERLINE: Rule = Rule {
    name: "underline",
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'t, Element<'t>> {
    debug!(log, "Trying to create underline container");

    assert_eq!(
        parser.current().token,
        Token::Underline,
        "Opening token isn't underline",
    );
    parser.step()?;

    collect_container(
        log,
        parser,
        RULE_UNDERLINE,
        ContainerType::Underline,
        &[ParseCondition::current(Token::Underline)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::token_pair(Token::Underline, Token::Whitespace),
            ParseCondition::token_pair(Token::Whitespace, Token::Underline),
        ],
    )
}
