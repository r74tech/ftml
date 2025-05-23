/*
 * parsing/rule/impls/block/parser.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2025 Wikijump Team
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

use super::arguments::Arguments;
use super::BlockRule;
use crate::parsing::collect::{collect_text, collect_text_keep};
use crate::parsing::condition::ParseCondition;
use crate::parsing::consume::consume;
use crate::parsing::{
    gather_paragraphs, parse_string, ExtractedToken, ParseError, ParseErrorKind,
    ParseResult, Parser, Token,
};
use crate::tree::Element;
use once_cell::sync::Lazy;
use regex::Regex;

static ARGUMENT_KEY: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z0-9_\-]+").unwrap());

impl<'r, 't> Parser<'r, 't>
where
    'r: 't,
{
    pub fn get_block_name(
        &mut self,
        flag_star: bool,
    ) -> Result<(&'t str, bool), ParseError> {
        debug!("Looking for identifier");

        if flag_star {
            self.get_optional_token(Token::LeftBlockStar)?;
        } else {
            self.get_optional_token(Token::LeftBlock)?;
        }

        self.get_optional_space()?;

        // Collect block name and determine whether the head is done
        self.get_block_name_internal(ParseErrorKind::BlockMissingName)
    }

    fn get_block_name_internal(
        &mut self,
        kind: ParseErrorKind,
    ) -> Result<(&'t str, bool), ParseError> {
        collect_text_keep(
            self,
            self.rule(),
            &[
                ParseCondition::current(Token::Whitespace),
                ParseCondition::current(Token::LineBreak),
                ParseCondition::current(Token::ParagraphBreak),
                ParseCondition::current(Token::RightBlock),
            ],
            &[],
            Some(kind),
        )
        .map(|(name, last)| {
            let name = name.trim();
            let in_head = match last.token {
                Token::Whitespace | Token::LineBreak | Token::ParagraphBreak => true,
                Token::RightBlock => false,

                // collect_text_keep() already checked the token
                _ => unreachable!(),
            };

            (name, in_head)
        })
    }

    /// Matches an ending block, returning the name present.
    pub fn get_end_block(&mut self) -> Result<&'t str, ParseError> {
        debug!("Looking for end block");

        self.get_token(Token::LeftBlockEnd, ParseErrorKind::BlockExpectedEnd)?;
        self.get_optional_space()?;

        let (name, in_head) = self.get_block_name(false)?;
        if in_head {
            self.get_optional_space()?;
            self.get_token(Token::RightBlock, ParseErrorKind::BlockExpectedEnd)?;
        }

        Ok(name)
    }

    /// Consumes an entire block end, validating that the newline and names match.
    ///
    /// Used internally by the body parsing methods.
    fn verify_end_block(
        &mut self,
        first_iteration: bool,
        block_rule: &BlockRule,
    ) -> Option<&'r ExtractedToken<'t>> {
        self.save_evaluate_fn(|parser| {
            // Check that the end block is on a new line, if required
            if block_rule.accepts_newlines {
                // Only check after the first, to permit empty blocks
                if !first_iteration {
                    parser.get_optional_line_break()?;
                }
            }

            // Check if it's an end block
            //
            // This will ignore any errors produced,
            // since it's just more text
            let name = parser.get_end_block()?;

            // Remove underscore for score flag
            let name = name.strip_suffix('_').unwrap_or(name);

            // Check if it's valid
            for end_block_name in block_rule.accepts_names {
                if name.eq_ignore_ascii_case(end_block_name) {
                    return Ok(true);
                }
            }

            Ok(false)
        })
    }

    // Body parsing

    /// Generic helper function that performs the primary block collection.
    ///
    /// Extended by the other, more specific functions.
    fn get_body_generic<F>(
        &mut self,
        block_rule: &BlockRule,
        mut process: F,
    ) -> Result<(&'r ExtractedToken<'t>, &'r ExtractedToken<'t>), ParseError>
    where
        F: FnMut(&mut Parser<'r, 't>) -> Result<(), ParseError>,
    {
        trace!("Running generic in block body parser");

        debug_assert!(
            !block_rule.accepts_names.is_empty(),
            "List of valid end block names is empty, no success is possible",
        );

        // Keep iterating until we find the end.
        // Preserve parse progress if we've hit the end block.
        let mut first = true;
        let start = self.current();

        loop {
            let at_end_block = self.verify_end_block(first, block_rule);

            // If there's a match, return the last body token
            if let Some(end) = at_end_block {
                return Ok((start, end));
            }

            // Run the passed-in closure
            process(self)?;

            // Step and continue
            self.step()?;
            first = false;
        }
    }

    /// Collect a block's body to its end, as string slice.
    ///
    /// This requires that the has already been parsed using
    /// one of the "get argument" methods.
    ///
    /// The `accepts_newlines` argument designates whether this
    /// block assumes multiline construction (e.g. `[[div]]`, `[[code]]`)
    /// or not (e.g. `[[span]]`).
    pub fn get_body_text(
        &mut self,
        block_rule: &BlockRule,
    ) -> Result<&'t str, ParseError> {
        debug!("Getting block body as text (rule {})", block_rule.name);

        // State variables for collecting span
        let (start, end) = self.get_body_generic(block_rule, |_| Ok(()))?;
        let slice = self.full_text().slice_partial(start, end);
        Ok(slice)
    }

    #[inline]
    pub fn get_body_elements(
        &mut self,
        block_rule: &BlockRule,
        as_paragraphs: bool,
    ) -> ParseResult<'r, 't, Vec<Element<'t>>> {
        debug!(
            "Getting block body as elements (block rule {}, as-paragraphs {})",
            block_rule.name, as_paragraphs,
        );

        if as_paragraphs {
            self.get_body_elements_paragraphs(block_rule)
        } else {
            self.get_body_elements_no_paragraphs(block_rule)
        }
    }

    fn get_body_elements_paragraphs(
        &mut self,
        block_rule: &BlockRule,
    ) -> ParseResult<'r, 't, Vec<Element<'t>>> {
        let mut first = true;

        gather_paragraphs(
            self,
            self.rule(),
            Some(move |parser: &mut Parser<'r, 't>| {
                let result = parser.verify_end_block(first, block_rule);
                first = false;

                Ok(result.is_some())
            }),
        )
    }

    fn get_body_elements_no_paragraphs(
        &mut self,
        block_rule: &BlockRule,
    ) -> ParseResult<'r, 't, Vec<Element<'t>>> {
        let mut all_elements = Vec::new();
        let mut all_errors = Vec::new();
        let mut paragraph_safe = true;
        let mut first = true;

        loop {
            let result = self.verify_end_block(first, block_rule);
            if result.is_some() {
                return ok!(paragraph_safe; all_elements, all_errors);
            }

            first = false;
            let old_remaining = self.remaining();
            let elements = consume(self)?.chain(&mut all_errors, &mut paragraph_safe);
            all_elements.extend(elements);

            // Step if the rule hasn't moved the pointer itself
            if self.same_pointer(old_remaining) {
                self.step()?;
            }
        }
    }

    // Block head / argument parsing
    pub fn get_head_map(
        &mut self,
        block_rule: &BlockRule,
        in_head: bool,
    ) -> Result<Arguments<'t>, ParseError> {
        trace!("Looking for key value arguments, then ']]'");

        let mut map = Arguments::new();
        if in_head {
            // Only process if the block isn't done yet
            loop {
                self.get_optional_spaces_any()?;

                // Try to get the argument key
                // Allows any token that matches the regular expression
                // i.e., alphanumeric, dash, or underscore
                //
                // This logic determines if we stop or keep getting arguments
                //
                // We could use collect_text_keep() here, but it messes with
                // get_head_block() so we just have it inline. Also it's a bit
                // strange since one of the outcomes is to break out of the loop.

                let key = {
                    let start = self.current();
                    let mut args_finished = false;

                    loop {
                        let current = self.current();
                        match current.token {
                            // End parsing block head
                            Token::RightBlock => {
                                args_finished = true;
                                break;
                            }

                            // End parsing argument key
                            Token::Whitespace
                            | Token::LineBreak
                            | Token::ParagraphBreak
                            | Token::Equals => break,

                            // Continue iterating to gather key
                            _ if ARGUMENT_KEY.is_match(current.slice) => {
                                self.step()?;
                            }

                            // Invalid token
                            _ => {
                                return Err(self
                                    .make_err(ParseErrorKind::BlockMalformedArguments))
                            }
                        }
                    }

                    // Stop iterating for more argument key-value pairs
                    if args_finished {
                        break;
                    }

                    // Gather argument key string slice
                    let end = self.current();
                    self.full_text().slice_partial(start, end)
                };

                // Equal sign
                self.get_optional_space()?;
                self.get_token(Token::Equals, ParseErrorKind::BlockMalformedArguments)?;

                // Get the argument value
                self.get_optional_space()?;
                let value_raw = self
                    .get_token(Token::String, ParseErrorKind::BlockMalformedArguments)?;

                // Parse the string
                let value = parse_string(value_raw);

                // Add to argument map
                map.insert(key, value);
            }
        }

        self.get_head_block(block_rule, in_head)?;
        Ok(map)
    }

    pub fn get_head_name_map(
        &mut self,
        block_rule: &BlockRule,
        in_head: bool,
    ) -> Result<(&'t str, Arguments<'t>), ParseError> {
        trace!("Looking for a name, then key value arguments, then ']]'");

        if !in_head {
            warn!("Block is already over, there is no name or arguments");
            return Err(self.make_err(ParseErrorKind::BlockMissingName));
        }

        // Get module's name
        let (subname, in_head) =
            self.get_block_name_internal(ParseErrorKind::ModuleMissingName)?;

        // Get arguments and end of block
        let arguments = self.get_head_map(block_rule, in_head)?;

        Ok((subname, arguments))
    }

    pub fn get_head_value<F, T>(
        &mut self,
        block_rule: &BlockRule,
        in_head: bool,
        convert: F,
    ) -> Result<T, ParseError>
    where
        F: FnOnce(&Self, Option<&'t str>) -> Result<T, ParseError>,
    {
        debug!("Looking for a value argument, then ']]' (in-head {in_head})");

        let argument = if in_head {
            // Gather slice of tokens in value
            let slice = collect_text(
                self,
                self.rule(),
                &[ParseCondition::current(Token::RightBlock)],
                &[
                    ParseCondition::current(Token::ParagraphBreak),
                    ParseCondition::current(Token::LineBreak),
                ],
                Some(ParseErrorKind::BlockMalformedArguments),
            )?;

            Some(slice)
        } else {
            None
        };

        // Convert the value into a type of the caller's choosing
        let value = convert(self, argument)?;

        // Set to false because the collection will always end the block
        self.get_head_block(block_rule, false)?;
        Ok(value)
    }

    pub fn get_head_none(
        &mut self,
        block_rule: &BlockRule,
        in_head: bool,
    ) -> Result<(), ParseError> {
        debug!("No arguments, looking for end of head block");
        self.get_optional_space()?;
        self.get_head_block(block_rule, in_head)?;
        Ok(())
    }

    // Helper function to finish up the head block
    fn get_head_block(
        &mut self,
        block_rule: &BlockRule,
        in_head: bool,
    ) -> Result<(), ParseError> {
        trace!("Getting end of the head block");

        // If we're still in the head, finish
        if in_head {
            self.get_token(Token::RightBlock, ParseErrorKind::BlockMissingCloseBrackets)?;
        }

        // If the block wants a newline after, take it
        //
        // It's fine if we're at the end of the input,
        // it could be an empty block type.
        if self.current().token != Token::InputEnd && block_rule.accepts_newlines {
            self.get_optional_line_break()?;
        }

        Ok(())
    }

    // Utilities
    #[inline]
    pub fn set_block(&mut self, block_rule: &BlockRule) {
        debug!("Running block rule {} for these tokens", block_rule.name);
        self.set_rule(block_rule.rule());
    }
}
