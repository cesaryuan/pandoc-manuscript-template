Attribute VB_Name = "MTConvertEquations"
'MTConvertEquations: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTConvertEquations.bas 54    1/24/14 10:10a Jimm $
'=====================================================================

'We turn off Smart Cut and Paste to stop Word removing space characters
'around equations, and also turn off Typing Replaces Selection.
'All original settings are restored at the end of the macro.

Option Explicit

'allow specified variables to be passed between procedures and from dialogs
Public gDlgCanceled As Boolean
Public gFindMathType As Boolean
Public gFindFields As Boolean
Public gFindText As Boolean
Public gFindOMML As Boolean
Public gPrompt As Boolean
Public gUpdateRange As Long
Public gTransOptions As Long
Public gTransName$

Public Sub DlgMain()
    Dim title As String
    Dim count As Long

    title = MTLib.GetUserString("!0100Convert Equations")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo bye
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo bye
    End If

    gTransName$ = ""

    'show dialog and let it disappear
    MTConvertEquationsDlg.Show
    DoEvents

    If Not gDlgCanceled Then
        ConvertEquations title, showStats:=True, count:=count
        'update statistics counters
        MTIncrementStatisticBy "CCvtEq", 1      ' user ran Convert Equations
        MTIncrementStatisticBy "CvtEq", count   ' total number of equations converted

        'convert from counts
        If gFindMathType Then
            MTIncrementStatisticBy "CvtEqFromMathType", 1 ' convert from MathType
        End If
        If gFindFields Then
            MTIncrementStatisticBy "CvtEqFromEQFields", 1 ' convert from Word EQ fields
        End If
        If gFindText Then
            MTIncrementStatisticBy "CvtEqFromText", 1 ' convert from text equations
        End If
        If gFindOMML Then
            MTIncrementStatisticBy "CvtEqFromOMML", 1 ' convert from OMML equations
        End If
        
        'convert to counts
        If Len(gTransName$) = 0 Then
            MTIncrementStatisticBy "CvtEqToMathType", 1 ' convert to MathType
        ElseIf InStr(gTransName$, "TeX") > 0 Then
            MTIncrementStatisticBy "CvtEqToTeX", 1 ' convert to one of the TeX text equations
        ElseIf InStr(gTransName$, "MathML") > 0 Then
            MTIncrementStatisticBy "CvtEqToMML", 1 ' convert to one of the MathML text equations
        End If

    End If
bye:
End Sub

Public Sub DoConvertEquations(showStats As Boolean, equationTypes As Long, _
    selectionOnly As Boolean, promptUser As Boolean, _
    translatorName As String, translatorOptions As Long, count As Long)

    Dim title As String
    title = MTLib.GetUserString("!0100Convert Equations")

    gFindMathType = equationTypes And 1
    gFindFields = equationTypes And 2
    gFindText = equationTypes And 4
    gFindOMML = equationTypes And 8
    gPrompt = promptUser
    If selectionOnly Then
        gUpdateRange = mt_RANGE_SELECTION
    Else
        gUpdateRange = mt_RANGE_DOCUMENT
    End If
    gTransOptions = translatorOptions
    gTransName$ = translatorName

    ConvertEquations title, showStats, count

End Sub

'This sub does all the actual work.
'Shows statistics at end if showStats = True.
Public Sub ConvertEquations(title As String, showStats As Boolean, count As Long)
    Dim ans As Long
    Dim stat As Long
    Dim oldSmart As Boolean
    Dim oldReplaceSel As Boolean
    Dim origRange As Range
    Dim wordState As Long
    Dim ueInfo As UpdateInfo
    Dim fieldCodesOn As Boolean
    Dim saveView As WdViewType

    'save state and turn off field codes
    fieldCodesOn = ActiveWindow.View.ShowFieldCodes
    If fieldCodesOn Then
        ActiveWindow.View.ShowFieldCodes = False
    End If

    'make sure that mtstyle_CONVERTED_EQUATION exists
    ValidateConvertedStyle ActiveDocument

    'set the mouse pointer
    System.Cursor = wdCursorWait

    'clear doc as having eqns from other platform, mark as this platform
    MTLib.DeleteEqnPlatformProperties ActiveDocument
    MTLib.AddEqnPlatformProperty ActiveDocument

    'init counts
    MTLib.InitEqnCounts

    'init update info data
    ueInfo.id = 1
    ueInfo.prompt = gPrompt
    ueInfo.title = title
    ueInfo.update = True        'replace equations
    ueInfo.kind = kUE_GRAPHIC   'convert to OLE eqn

    On Error GoTo error1

    'save SmartCut&Paste setting(s) & TypingReplacesSelection settings, & turn them off
    wordState = MTLib.SaveWordState
    options.SmartCutPaste = False
    If Val(Application.version) >= kWordX Then
       MTLib.SetPasteSmartCutPaste False
    End If
    options.ReplaceSelection = False
    ActiveDocument.TrackRevisions = False
    saveView = ActiveWindow.View.Type
    #If Mac Then
    'the following option has to be disabled when converting equations:
    'Word\Preferences\Equations\General\Copy linear format to the clipboard as plain text
    Dim saveOMathCopyLF As Boolean
    saveOMathCopyLF = options.OMathCopyLF
    options.OMathCopyLF = False
    #End If

    'mark orig. location in doc.
    Set origRange = Selection.Range

    'reset any prefs/translators (lets eqns w/own prefs use them)
    stat = MTXFormReset()
    If stat = mtOK Then
        'send the new translator if necessary
        If gTransName$ <> "" Then
            'for text conversions, always append a 'Z' to end of text on clipboard
            'to workaround Word bug on Win 9x/XP where trailing CRLF is truncated on paste
            'pasting code in MTLib.CleanUpPasteKludge deletes this extra character
            gTransOptions = gTransOptions Or mtxfmTRANSL_INC_CLIPBOARD_EXTRA
            stat = MTXFormSetTranslator(gTransOptions, gTransName$)
            ueInfo.kind = kUE_TEXT
        End If
    End If

    If stat = mtOK Then
        'if not prompting, disable screen updates for better performance
        If Not ueInfo.prompt Then
            MTLib.SetScreenUpdate False
        End If

        'if converting to text-based eqns, convert existing text eqns first
        'else if converting to eqn objects, convert OLE equations first
        'this avoids updating these items twice
        ans = 0
        If ueInfo.kind = kUE_TEXT Then 'converting to text equations
            'update Text (TeX/MathML) equations if requested
            'convert text equations to text equations
            If gFindText Then
                ans = MTLib.UpdateTextEqns(gUpdateRange, ueInfo)
                'restore original selection
                If gUpdateRange = mt_RANGE_SELECTION Then
                    origRange.Select
                End If
            End If
            ' update OMML and OMML images to MT eqns which are then converted to text below
            If gFindOMML Then
                ans = MTLib.UpdateOMMLEqns(gUpdateRange, ueInfo)
                'restore original selection
                If gUpdateRange = mt_RANGE_SELECTION Then
                    origRange.Select
                End If
            End If
            'update OLE1, OLE2 & Non-OLE MathType Eqn Pictures selected if requested
            'convert MathType equations to text equations
            If gFindMathType And ans <> -1 Then
                ans = MTLib.UpdateGraphics(gUpdateRange, ueInfo)
                'restore original selection
                If gUpdateRange = mt_RANGE_SELECTION Then
                    origRange.Select
                End If
            End If
            'update "MathType 1.x macro equations" or "Microsoft Word formula fields" if requested
            If gFindFields And ans <> -1 Then
                ans = MTLib.UpdateFields(gUpdateRange, ueInfo)
            End If
        Else 'converting to MathType equations
            'update OLE1, OLE2 & Non-OLE MathType Eqn Pictures selected if requested
            'convert MathType equations to MathType equations
            If gFindMathType Then
                ans = MTLib.UpdateGraphics(gUpdateRange, ueInfo)
                'restore original selection
                If gUpdateRange = mt_RANGE_SELECTION Then
                    origRange.Select
                End If
            End If
            'convert OMML equations to MathType equations
            If gFindOMML Then
                ans = MTLib.UpdateOMMLEqns(gUpdateRange, ueInfo)
                'restore original selection
                If gUpdateRange = mt_RANGE_SELECTION Then
                    origRange.Select
                End If
            End If
            'update "MathType 1.x macro equations" or "Microsoft Word formula fields" if requested
            If gFindFields And ans <> -1 Then
                ans = MTLib.UpdateFields(gUpdateRange, ueInfo)
                'restore original selection
                If gUpdateRange = mt_RANGE_SELECTION Then
                    origRange.Select
                End If
            End If
            'update Text (TeX/MathML) equations if requested
            'convert text equations to MathType equations
            If gFindText And ans <> -1 Then
                ans = MTLib.UpdateTextEqns(gUpdateRange, ueInfo)
            End If
        End If

        'set the mouse pointer to the hourglass
        System.Cursor = wdCursorNormal

        're-enable screen updates
        MTLib.SetScreenUpdate True

        'clear the status bar
        Application.StatusBar = ""

        'display statistics about updated equations
        If showStats Then
            MTLib.StatBox _
                MTLib.GetUserString("!1200The following equations were updated:"), _
                title
        End If
    Else
        MsgBox MTLib.GetUserString("!1202There was an error setting up MathType to convert equations. No equations were converted."), vbCritical, MTLib.GetUserString("!1203Conversion Error")
    End If

    'restore the original selection
    origRange.Select

error1:
    'restore original options settings
    MTLib.RestoreWordState (wordState)

    'reset the clipboard & mouse pointer
    VBAEmptyClipboard
    System.Cursor = wdCursorNormal

    'restore field codes
    If fieldCodesOn Then
        ActiveWindow.View.ShowFieldCodes = True
    End If

    'restore view
    ActiveWindow.View.Type = saveView
    
    'restore When Copying an Equation setting
    #If Mac Then
    options.OMathCopyLF = saveOMathCopyLF
    #End If

    count = ueInfo.id - 1
End Sub
