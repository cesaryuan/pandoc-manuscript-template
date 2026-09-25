{-
  Long-lived Pandoc worker for the PMT HTML server.

  The worker parses PMT's fixed HTML defaults once, then accepts JSON lines
  containing an input Markdown path and an output path. It invokes Pandoc's
  Haskell API for each request, so Lua filters and the Pandoc runtime remain in
  memory while the existing external pandoc-crossref filter still runs with
  the same project environment.
-}

{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE OverloadedStrings #-}

module Main where

import Control.Exception (SomeException, try)
import Data.Aeson (FromJSON, Value, eitherDecode, encode, object, (.=))
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy.Char8 as BL8
import GHC.Generics (Generic)
import System.Directory (canonicalizePath, createDirectoryIfMissing, makeAbsolute)
import System.Environment (getArgs)
import System.FilePath (isRelative, makeRelative, normalise, takeDirectory, (</>))
import System.IO (BufferMode (LineBuffering), hSetBuffering, stdin, stdout)
import Text.Pandoc.App (Opt (optInputFiles, optOutputFile), convertWithOpts,
                        defaultOpts, options, parseOptionsFromArgs)
import Text.Pandoc.Lua (getEngine)
import Text.Pandoc.Scripting (ScriptingEngine)

data Config = Config
  { pandocArgs :: [String]
  , projectDir :: FilePath
  } deriving (Generic, Show)

instance FromJSON Config

data Request = Request
  { input :: FilePath
  , output :: FilePath
  } deriving (Generic, Show)

instance FromJSON Request

main :: IO ()
main = do
  hSetBuffering stdin LineBuffering
  hSetBuffering stdout LineBuffering
  args <- getArgs
  configPath <- argumentValue "--config" args
  config <- decodeFile configPath
  parsed <- parseOptionsFromArgs options defaultOpts "pmt-pandoc-worker" (pandocArgs config)
  baseOpts <- either (fail . show) pure parsed
  engine <- getEngine
  loop engine config baseOpts

loop :: ScriptingEngine -> Config -> Opt -> IO ()
loop engine config baseOpts = do
  -- Lazy ByteString Char8 has no line reader; convert each strict stdin line.
  line <- BL8.fromStrict <$> B8.getLine
  if BL8.null line
    then pure ()
    else do
      response <- case eitherDecode line of
        Left err -> pure $ object ["ok" .= False, "error" .= err]
        Right request -> convertRequest engine config baseOpts request
      BL8.putStrLn (encode response)
      loop engine config baseOpts

convertRequest :: ScriptingEngine -> Config -> Opt -> Request -> IO Value
convertRequest engine config baseOpts request = do
  result <- try $ do
    source <- resolveProjectPath (projectDir config) (input request)
    target <- resolveProjectPath (projectDir config) (output request)
    createDirectoryIfMissing True (takeDirectory target)
    let opts = baseOpts
          { optInputFiles = Just [source]
          , optOutputFile = Just target
          }
    convertWithOpts engine opts
  case result of
    Left err -> pure $ object ["ok" .= False, "error" .= show (err :: SomeException)]
    Right () -> pure $ object ["ok" .= True]

resolveProjectPath :: FilePath -> FilePath -> IO FilePath
resolveProjectPath root raw = do
  root' <- canonicalizePath root
  path <- normalise <$> makeAbsolute (if isRelative raw then root' </> raw else raw)
  let relative = makeRelative root' path
  -- Accept descendants while rejecting paths that escape the project via `..`.
  if relative == "." || (isRelative relative && not (".." `prefixOf` relative))
    then pure path
    else fail $ "Path is outside the PMT project: " ++ path

prefixOf :: String -> String -> Bool
prefixOf prefix value = take (length prefix) value == prefix

argumentValue :: String -> [String] -> IO String
argumentValue name args = case dropWhile (/= name) args of
  (_ : value : _) -> pure value
  _ -> fail $ "Missing " ++ name

decodeFile :: FromJSON a => FilePath -> IO a
decodeFile path = do
  contents <- BL8.readFile path
  either fail pure (eitherDecode contents)
