module Main where


import Prelude
import Web.Event.EventTarget

import Css as Css
import Css as Style
import Data.Array as Array
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.Newtype (class Newtype, over)
import Data.String as String
import Effect (Effect)
import Effect.Console (log)
import Grain (class LocalGrain, LProxy(..), VNode, fromConstructor, mount, useUpdater, useValue)
import Grain.Markup (aside)
import Grain.Markup as H
import Unsafe.Coerce (unsafeCoerce)
import Web.DOM.Element (toNode)
import Web.DOM.ParentNode (QuerySelector(..), querySelector)
import Web.Event.Event (EventType(..))
import Web.HTML (window)
import Web.HTML.HTMLDocument (toParentNode)
import Web.HTML.Window (document)


extention :: VNode
extention = H.component do
  pure ui
  where
    ui = H.div
      # H.css (Style.stylesheet
        [ Style.styles
          [ Style.style "display" "flex"
          , Style.style "justify-content" "center"
          , Style.style "align-items" "center"
          , Style.style "background-color" "#cecece21"
          , Style.style "margin" "4px"
          , Style.style "box-shadow" "0 0 1px 0px #000"
          , Style.style "margin-left" "calc(20px + 4px)"
          ]
        , Css.onHover
              [ Css.style "background-color" "#d2f5ff85"
              , Css.style "box-shadow" "0 0 1px 1px #4c4c4c52"
              ]
        ])
      # H.kids
        [ H.span
          # H.kids
            [ H.text "+"
            ]
        ]


newtype Section = Section
  { rows :: Int
  , columns :: Int
  }

instance localGrainSection :: LocalGrain Section where
  initialState _ = pure $ Section {rows: 1, columns: 1}
  typeRefOf _ = fromConstructor Section

data AsideType
  = TopAside
  | LeftAside
  | RightAside
  | BotAside

section :: VNode
section = H.component do
  Section count <- useValue (LProxy :: _ Section)
  updateCount <- useUpdater (LProxy :: _ Section)
  let inc ty = updateCount $ \(Section c) -> case ty of
        TopAside -> Section $ c {columns = if c.columns >= 4 then c.columns else c.columns + 1}
        LeftAside -> Section $ c {rows = if c.rows >= 20 then c.rows else c.rows + 1}
        RightAside -> Section $ c {rows = if c.rows <= 1 then 1 else c.rows - 1}
        BotAside -> Section $ c {columns = if c.columns <= 1 then 1 else c.columns - 1}
  pure $ ui inc count
  where
    ui inc count = H.div
      # H.css (Css.styles
            [ Css.displayGrid
            , Css.style "grid-template-areas"
                ("\"top-left-gutter top top-right-gutter\" \"left main right\" \"bot-left-gutter bot bot-right-gutter\"")
            , Css.style "grid-template-rows" "20px 1fr 20px"
            , Css.style "grid-template-columns" "20px 1fr 20px"
            , Css.style "height" "100%"
            , Css.style "padding" "4px"
            ])
      # H.prop "section" ""
      # H.kids
        [ gutter "top-left-gutter"
        , topAside inc count
        , gutter "top-right-gutter"
        , leftAside inc count
        , panels inc count
        , gutter "bot-left-gutter"
        , rightAside inc count
        , botAside inc count
        , gutter "bot-right-gutter"
        ]
    gutter ty = H.div
      # H.css
        ( Css.stylesheet
        [ Css.styles
          [ Css.style "grid-area" ty
          , Css.style "display" "block"
          ]
        ])
    topAside inc count = H.div
      # H.css
        ( Css.stylesheet
        [ Css.styles
          [ Css.displayFlex
          , Css.style "grid-area" "top"
          , Css.style "width" "100%"
          , Css.style "padding" "4px"
          , Css.style "grid-column-gap" "4px"
          , Css.style "padding-left" "0"
          , Css.style "padding-right" "0"
          ]
        , Css.mediaQuery "max-width" "900px"
            [ Css.styles
              [Css.style "display" "none"]
            ]
        ])
      # H.prop "top" ""
      # H.prop "section-control" "top"
      # H.onClick (const $ inc TopAside)
      # H.kids (Array.replicate count.columns $ asideDivider TopAside)
    leftAside inc count = H.div
      # H.css
        ( Css.stylesheet
          [ Css.styles
            [ Css.style "grid-area" "left"
            , Css.style "display" "flex"
            , Css.style "flex-direction" "column"
            , Css.style "padding" "0 6px"
            , Css.style "grid-row-gap" "4px"
            ]
          , Css.mediaQuery "max-width" "900px"
            [ Css.styles
              [Css.style "display" "none"]
            ]
          ])
      # H.prop "left" ""
      # H.prop "section-control" "left"
      # H.onClick (const $ inc LeftAside)
      # H.kids (Array.replicate count.rows $ asideDivider LeftAside)
    panels inc count = H.div
      # H.css (Style.stylesheet
        [ Css.styles
          [ Css.displayGrid
          , Css.style "grid-area" "main"
          , Css.style "grid-column-gap" "4px"
          , Css.style "grid-row-gap" "4px"
          , Css.style "grid-template-columns" ("repeat(" <> show count.columns <>", 1fr)")
          ]
        , Css.mediaQuery "max-width" "900px"
            [ Css.styles
              [ Css.style "grid-template-columns" "1fr"
              ]
            ]
        ])
      # H.prop "main" ""
      # H.kids (panelChildren inc count)
    panelChildren inc count = Array.replicate
      (count.rows * count.columns)
      (panelChild inc count)
    panelChild inc count = H.div
      # H.css (Style.stylesheet
        [ Style.styles
          [ Css.style "background-color" "#dedede"
          , Css.style "min-height" "200px"
          , Css.style "min-width" "20px"
          ]
        ])
    rightAside inc count = H.div
      # H.css
      ( Css.stylesheet
        [ Css.styles
          [ Css.style "grid-area" "right"
          , Css.style "display" "flex"
          , Css.style "flex-direction" "column"
          , Css.style "padding" "0 6px"
          , Css.style "grid-row-gap" "4px"
          ]
        , Css.mediaQuery "max-width" "900px"
            [ Css.styles
              [Css.style "display" "none"]
            ]
        ])
      # H.prop "right" ""
      # H.prop "section-control" "right"
      # H.onClick (const $ inc RightAside)
      # H.kids (Array.replicate count.rows $ asideDivider RightAside)
    botAside inc count = H.div
      # H.css
        ( Css.stylesheet
        [ Css.styles
          [ Css.displayFlex
          , Css.style "grid-area" "bot"
          , Css.style "padding" "4px"
          , Css.style "grid-column-gap" "4px"
          , Css.style "padding-left" "0"
          , Css.style "padding-right" "0"
          ]
        , Css.mediaQuery "max-width" "900px"
            [ Css.styles
              [Css.style "display" "none"]
            ]
        ])
      # H.prop "bot" ""
      # H.prop "section-control" "bot"
      # H.onClick (const $ inc BotAside)
      # H.kids (Array.replicate count.columns $ asideDivider BotAside)
    asideDivider asideType = H.div
        # H.css
          ( Css.stylesheet
          [ Css.styles
            [ Css.displayFlex
            , Css.style "background-color" "#cecece21"
            , Css.style "margin" "0"
            , Css.style "box-shadow" "0 0 1px 0px #000"
            , Css.style "justify-content" "center"
            , Css.style "align-items" "center"
            , Css.style "user-select" "none"
            , case asideType of
                LeftAside ->  Css.style "height" "100%"
                RightAside ->  Css.style "height" "100%"
                TopAside -> Css.style "width" "100%"
                BotAside -> Css.style "width" "100%"
            , case asideType of
                TopAside -> Css.style "background-color" "#48d69b2e"
                LeftAside ->  Css.style "background-color" "#48d69b2e"
                RightAside ->  Css.style "background-color" "#d648482e"
                BotAside -> Css.style "background-color" "#d648482e"
            ]
          , Css.onHover
            [ Css.style "background-color" "#d2f5ff85"
            , Css.style "box-shadow" "0 0 1px 1px #4c4c4c52"
            ]
          ])
        # H.kids (case asideType of
            TopAside -> [H.text "+"]
            BotAside -> [H.text "-"]
            LeftAside -> [H.text "+"]
            RightAside -> [H.text "-"])




view :: VNode
view = H.div
  # H.css (Style.styles
      [ Style.style "display" "flex"
      , Style.style "flex-direction" "column"
      , Css.style "min-height" "100%"
      ])
  # H.kids
    [ extention
    , section
    , extention
    ]



main :: Effect Unit
main = do
  win <- window
  callback <- eventListener f
  addEventListener (EventType "load") callback false (unsafeCoerce win)
  where
    f :: _ -> Effect Unit
    f _ = do
      maybeEl <- window >>= document <#> toParentNode >>= querySelector (QuerySelector "body")
      case maybeEl of
        Nothing -> pure unit
        Just el ->
          mount view $ toNode el






