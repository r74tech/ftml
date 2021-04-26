/*
 * parsing/rule/impls/block/blocks/checkbox.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

pub const BLOCK_CHECKBOX: BlockRule = BlockRule {
    name: "block-checkbox",
    accepts_names: &["checkbox"],
    accepts_special: true,
    accepts_modifier: false,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    modifier: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing checkbox block";
        "in-head" => in_head,
        "name" => name,
        "special" => special,
    );

    assert!(!modifier, "Checkbox doesn't allow modifier variant");
    assert_block_name(&BLOCK_CHECKBOX, name);

    let arguments = parser.get_head_map(&BLOCK_CHECKBOX, in_head)?;
    parser.get_optional_space()?;

    let element = Element::CheckBox {
        checked: special,
        attributes: arguments.to_hash_map(),
    };

    ok!(element)
}
