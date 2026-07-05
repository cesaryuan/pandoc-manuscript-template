Attribute VB_Name = "MTFormatEquations"
'MTFormatEquations: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTFormatEquations.bas 51    1/24/14 10:10a Jimm $
'=====================================================================

'Reformats all equations in selection or whole doc. User can select prefs to use.
'All equations become MathType OLE objects.
'Marks doc with 'has this platform' type eqns, removes other platform props.

Option Explicit

'globals for passing data between procedures and dialog
Public gMTPrefs$
Public gMTPrefsSource$
Public gSavePrefs As Boolean
Public gUpdateRange As Long
Public gDlgCanceled As Boolean

Public Sub DlgMain()
    Dim title As String
    Dim count As Long

    On Error GoTo abort

    title = MTLib.GetUserString("!0500Format Equations")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If

    'run the dialog & allow to disappear
    MTFormatEqn.Show
    DoEvents

    If Not gDlgCanceled Then
        FormatEquations title, showStats:=True, count:=count
        MTIncrementStatisticBy "CFmtEq", 1
        MTIncrementStatisticBy "FmtEq", count
    End If

abort:
End Sub

Public Sub DoFormatEquations(showStats As Boolean, prefsSource As Long, _
        prefsFileName As String, selectionOnly As Boolean, _
        savePrefs As Boolean, count As Long)

    Dim title As String
    title = MTLib.GetUserString("!0500Format Equations")

    Select Case prefsSource
    Case 1
        gMTPrefs$ = MTLib.GetPrefsFromDoc$
        If gMTPrefs$ = "" Then GoTo err
        gMTPrefsSource$ = ""
    Case 2
        gMTPrefs$ = MTLib.GetPrefsFromMType$
        If MTFormatEquations.gMTPrefs$ = "" Then GoTo err
        gMTPrefsSource$ = MTLib.GetUserString2("0810", "2410", "MathType")
    Case 3
        gMTPrefs$ = MTLib.GetPrefsFromClipboard$
        If gMTPrefs$ = "" Then GoTo err
        gMTPrefsSource$ = MTLib.GetUserString2("0809", "2409", "clipboard")
    Case 4
        If prefsFileName = "" Then
            MsgBox MTLib.GetUserString("!0514You must choose a preference file."), _
                vbExclamation, title
            GoTo err
        Else
            gMTPrefs$ = MTLib.GetPrefsFromFile$(prefsFileName)
            If gMTPrefs$ = "" Then GoTo err
            gMTPrefsSource$ = MTLib.GetFileNameFromPath$(prefsFileName)
        End If
    Case Else
        GoTo err
    End Select

    If selectionOnly Then
        gUpdateRange = mt_RANGE_SELECTION
    Else
        gUpdateRange = mt_RANGE_DOCUMENT
    End If
    gSavePrefs = savePrefs

    FormatEquations title, showStats, count
err:
End Sub

'Does the real work
Public Sub FormatEquations(title As String, showStats As Boolean, count As Long)
    Dim stat As Long
    Dim wordState As Long
    Dim origRange As Range
    Dim ueInfo As UpdateInfo
    Dim Doc As Document
    Dim saveView As WdViewType

    'save preferences in document
    Set Doc = ActiveDocument
    If gSavePrefs And gMTPrefs$ <> "" Then
        MTLib.WriteDocPropString Doc, mtprop_PREFERENCES, gMTPrefs$
        If gMTPrefsSource$ <> "" Then
            MTLib.WriteDocPropString Doc, mtprop_PREFERENCES_FILE, gMTPrefsSource$
        End If

        'delete "Use MT's defaults" property so we'll use these prefs.
        MTLib.DeleteDocProperty Doc, mtprop_USE_MATHTYPE_PREFS
    End If

    'mark doc as having eqns from this platform
    MTLib.DeleteEqnPlatformProperties Doc
    MTLib.AddEqnPlatformProperty Doc

    'init counts
    MTLib.InitEqnCounts

    ueInfo.id = 1
    ueInfo.prompt = False
    ueInfo.title = title
    ueInfo.update = True
    ueInfo.kind = kUE_GRAPHIC

    On Error GoTo error1

    'save SmartCut&Paste & TypingReplacesSelection settings, & turn them off
    wordState = MTLib.SaveWordState
    options.SmartCutPaste = False
    If Val(Application.version) >= kWordX Then
       MTLib.SetPasteSmartCutPaste False
    End If
    options.ReplaceSelection = False
    ActiveDocument.TrackRevisions = False
    saveView = ActiveWindow.View.Type

    'remember the original selection
    Set origRange = Selection.Range

    'reset prefs/translators (lets eqns w/own prefs use them)
    stat = MTXFormReset()

    If stat = mtOK Then
        'send the preferences
        If gMTPrefs$ <> "" Then
            stat = MTXFormSetPrefs(mtxfmPREF_USER, gMTPrefs$)
        Else
            stat = MTXFormSetPrefs(mtxfmPREF_EXISTING, "")
        End If
    End If

    If stat = mtOK Then
        MTLib.SetScreenUpdate False

        System.Cursor = wdCursorWait

        'update OLE eqns/pictures
        stat = MTLib.UpdateGraphics(gUpdateRange, ueInfo)

        'update MT 1.x macro eqns & Word formula fields
        If stat = 0 Then
            If gUpdateRange = mt_RANGE_SELECTION Then
                origRange.Select
            End If
            stat = MTLib.UpdateFields(gUpdateRange, ueInfo)
        End If

        If gUpdateRange = mt_RANGE_SELECTION Then
            origRange.Select
        End If

        System.Cursor = wdCursorNormal
        MTLib.SetScreenUpdate True
        DoEvents
        Application.StatusBar = ""
    Else
        MsgBox MTLib.GetUserString("!1402There was an error setting up MathType to format equations. No equations were reformatted."), _
            vbCritical, title
    End If

    origRange.Select
    MTLib.RestoreWordState (wordState)

    If showStats Then
        MTLib.StatBox MTLib.GetUserString("!1400The following equations were updated:"), title
    End If

error1:
    VBAEmptyClipboard
    System.Cursor = wdCursorNormal
    'restore view
    ActiveWindow.View.Type = saveView
    count = ueInfo.id - 1
End Sub




