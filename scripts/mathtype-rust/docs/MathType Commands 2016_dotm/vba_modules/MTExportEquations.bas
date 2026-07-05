Attribute VB_Name = "MTExportEquations"
'MTExportEquations 5.0
'====================================================================
' (c) Copyright 2001-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTExportEqns.bas 21    1/24/14 10:10a Jimm $
'====================================================================
Option Explicit

Type ExportEquationDlgInfo
    selType As Long         'in: type of selection
    replace As Boolean      'in/out: true to replace eqns with filename
    fileType As Long        'in/out 0-based ID of filetype
    path As String          'in/out: directory for saving
    pattern As String       'in/out: filename pattern
    start As Long           'in/out: starting #
    deleteAll As Boolean    'in/out: true to delete all files in target dir with same extension
    rangeType As Long       'out: mt_RANGE_SELECTION/mt_RANGE_DOCUMENT
    dlgCanceled As Boolean  'out: true if cancelled
End Type

#If Win32 Then
Private Const kMaxFileTypeID As Long = 4
#Else
Private Const kMaxFileTypeID As Long = 5
#End If

Public gExportEquationDlgInfo As ExportEquationDlgInfo

Public Sub DlgMain()
    Dim title As String
    Dim count As Long

    'check to see if a document is open
    title = MTLib.GetUserString("!0610Export Equations")
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo bye
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo bye
    End If

    'Put up the dialog and get results
    With gExportEquationDlgInfo
        .selType = Selection.Type
        .replace = GetDefaultExportReplace()
        .pattern = GetDefaultExportPattern()
        .fileType = GetDefaultExportFileType()
        .start = 1
        .deleteAll = False
        .path = GetDefaultExportDir()
    End With
    MTExportEqns.Show

    'convert if "OK" button pressed
    If Not gExportEquationDlgInfo.dlgCanceled Then
        ExportEquations title, showStats:=True, count:=count
        'update statistics counters
        MTIncrementStatisticBy "CExpEq", 1
        MTIncrementStatisticBy "ExpEq", count
        With gExportEquationDlgInfo
            Select Case .fileType
            Case kFTEPS_OSPICT, kFTEPS_TIFF, kFTEPS_NONE
                MTIncrementStatisticBy "ExpEqEPS", 1
            Case kFTGIF
                MTIncrementStatisticBy "ExpEqGIF", 1
            Case kFTOSPICT
                #If Win32 Then
                MTIncrementStatisticBy "ExpEqWMF", 1
                #Else
                MTIncrementStatisticBy "ExpEqPICT", 1
                #End If
            Case kFTPDF
                MTIncrementStatisticBy "ExpEqPDF", 1
            End Select
            If .replace Then
                MTIncrementStatisticBy "ExpEqRpl", 1
            End If
        End With
    End If

bye:
End Sub

'Returns default pattern
Private Function GetDefaultExportPattern() As String
    GetDefaultExportPattern = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_PATTERN)
    If GetDefaultExportPattern = "" Then
        GetDefaultExportPattern = "Eqn###"
    End If
End Function

'Returns default replace setting
Private Function GetDefaultExportReplace() As Boolean
    GetDefaultExportReplace = (GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_REPLACE) = "1")
End Function

'Returns default replace setting
Private Function GetDefaultExportDir() As String
    GetDefaultExportDir = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_DIRECTORY)
End Function

'Returns default filetype
Private Function GetDefaultExportFileType() As Long
    Dim value As Long
    value = CLng(Val(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_FILETYPE)))
    If value < 0 Or value > kMaxFileTypeID Then
        GetDefaultExportFileType = 0
    Else
        GetDefaultExportFileType = value
    End If
End Function

Public Sub DoExportEquations(showStats As Boolean, _
    folderPath As String, deleteAll As Boolean, fileType As Long, _
    fileNamePattern As String, firstNumber As Long, replace As Boolean, _
    selectionOnly As Boolean, count As Long)

    Dim title As String
    title = MTLib.GetUserString("!0610Export Equations")

    With MTExportEquations.gExportEquationDlgInfo
        .selType = Selection.Type
        .replace = replace
        .fileType = fileType
        .path = folderPath
        .pattern = fileNamePattern
        .start = firstNumber
        .deleteAll = deleteAll
        If selectionOnly Then
            .rangeType = mt_RANGE_SELECTION
        Else
            .rangeType = mt_RANGE_DOCUMENT
        End If
        .dlgCanceled = False
    End With

    ExportEquations title, showStats, count
End Sub

Public Sub ExportEquations(title As String, showStats As Boolean, count As Long)
    Dim stat As Long
    Dim wordState As Long
    Dim origRange As Range
    Dim ueInfo As UpdateInfo
    Dim value As String
    Dim saveView As WdViewType

    'save settings (not 'deleteAll')
    With gExportEquationDlgInfo
        If .replace Then
            value = "1"
        Else
            value = "0"
        End If
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_REPLACE, value
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_PATTERN, .pattern
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_FILETYPE, Strings.Trim(Conversion.str$(.fileType))
        SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_EXPORT_DIRECTORY, .path

        If .deleteAll Then
            'delete all files with matching extension
            On Error Resume Next
            Kill .path & Application.PathSeparator & "*" & GetFileTypeExtension(.fileType)
        End If
    End With

    'init counts
    MTLib.InitEqnCounts

    'setup info structure
    With ueInfo
        .id = gExportEquationDlgInfo.start
        .prompt = False
        .title = title
        .update = gExportEquationDlgInfo.replace
        .kind = kUE_FILE
        .fileName = gExportEquationDlgInfo.path
        'strip any trailing slash off the folder name
        If Strings.right$(.fileName, 1) = "\" Then
            .fileName = Strings.left$(.fileName, Len(.fileName) - 1)
        End If
        .filePattern = gExportEquationDlgInfo.pattern & _
            GetFileTypeExtension(gExportEquationDlgInfo.fileType)
        .fileType = gExportEquationDlgInfo.fileType
    End With

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

    'reset prefs/translators, let eqns w/own prefs use them
    If MTXFormReset() = mtOK Then
        MTLib.SetScreenUpdate False
        System.Cursor = wdCursorWait

        'update OLE eqns/pictures
        stat = MTLib.UpdateGraphics(gExportEquationDlgInfo.rangeType, ueInfo)

        If gExportEquationDlgInfo.rangeType = mt_RANGE_SELECTION Then
            origRange.Select
        End If

        System.Cursor = wdCursorNormal
        MTLib.SetScreenUpdate True
        DoEvents
        Application.StatusBar = ""
    Else
        MsgBox MTLib.GetUserString("!1451There was an error setting up MathType to export equations. No equations were exported."), _
            vbCritical, ueInfo.title
    End If

    origRange.Select
    MTLib.RestoreWordState (wordState)
    ActiveWindow.View.Type = saveView

    If showStats Then
        MTLib.StatBox MTLib.GetUserString("!1450The following equations were exported:"), ueInfo.title
    End If

error1:
    VBAEmptyClipboard
    System.Cursor = wdCursorNormal

    count = ueInfo.id - gExportEquationDlgInfo.start
End Sub

'Calculates filename from pattern & file#
Public Function GetFileNameFromPattern(pattern As String, id As Long) As String
    Dim pos As Long, patternLen As Long
    Dim fileName As String
    Dim idStr As String, idLen As Long

    'get pos of first '#'
    pos = InStr(1, pattern, "#", vbBinaryCompare)

    'if no '#', just use pattern as filename
    If pos <= 0 Then
        GetFileNameFromPattern = pattern
        Exit Function
    End If

    'get number of consecutive '#'s
    patternLen = 1
    While (Strings.Mid$(pattern, pos + patternLen, 1) = "#")
        patternLen = patternLen + 1
    Wend

    'get id as string
    idStr = Strings.Trim$(Conversion.str(id))
    idLen = Len(idStr)

    'pad with '0's if necessary
    fileName = Strings.left$(pattern, pos - 1)
    If idLen < patternLen Then
        fileName = fileName & Strings.String(patternLen - idLen, "0")
    End If

    'append id & remainder of pattern
    GetFileNameFromPattern = fileName & idStr & Strings.Mid$(pattern, pos + patternLen)

End Function

'Returns extension for fileType, depends upon specific order of items in list
Public Function GetFileTypeExtension(fileType As Long) As String
    Select Case fileType
    Case kFTEPS_OSPICT, kFTEPS_TIFF, kFTEPS_NONE
        GetFileTypeExtension = ".eps"
    Case kFTGIF
        GetFileTypeExtension = ".gif"
    Case kFTOSPICT
        #If Win32 Then
        GetFileTypeExtension = ".wmf"
        #Else
        GetFileTypeExtension = ".pict"
        #End If
    Case 5
        GetFileTypeExtension = ".pdf"
    End Select
End Function




