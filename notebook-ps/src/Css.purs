module Css
  ( stylesheet
  , styles
  , onHover
  , mediaQuery
  , style
  , displayFlex
  , displayGrid
  , backgroundColor
  , firstChild
  , lastChild
  ) where

import Prelude
import Data.String as String

type CssStyle = String

stylesheet :: Array String -> String
stylesheet = String.joinWith ""

styles :: Array CssStyle -> String
styles xs = ".& {" <> String.joinWith "" xs <> "}"

onHover :: Array CssStyle -> String
onHover xs = ".&:hover {" <> String.joinWith "" xs <> "}"

mediaQuery :: String -> String -> Array CssStyle -> String
mediaQuery l r xs = "@media only screen and (" <> l <> ":" <> r <> ") {" <> String.joinWith "" xs <> "}"

firstChild :: Array CssStyle -> String
firstChild xs = ".&:first-child {" <> String.joinWith "" xs <> "}"

lastChild :: Array CssStyle -> String
lastChild xs = ".&:last-child {" <> String.joinWith "" xs <> "}"

style :: String -> String -> CssStyle
style x y = x <> ":" <> y <> ";"

backgroundColor :: String -> CssStyle
backgroundColor = style "background-color"


displayFlex :: CssStyle
displayFlex = style "display" "flex"

displayGrid :: CssStyle
displayGrid = style "display" "grid"

