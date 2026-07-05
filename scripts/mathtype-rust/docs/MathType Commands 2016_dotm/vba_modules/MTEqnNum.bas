Attribute VB_Name = "MTEqnNum"
'MTEqnNum 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTEqnNum.bas 52    5/06/14 9:54a Jimm $
'=====================================================================

'Places an equation number at insertion point with current format.
'in the custom document property "MTEquationNumber". If this property
'does not exist the default is the style "(1.1)". Also check for
'MTW3's AutoText entry "ZMTEqnNumFormatPrefs".
'The # is enclosed in a MacroButton field, so when double-clicked the
'MTPlaceRef places an reference to the equation number at the location
'marked by a bookmark created by MTMarkRef (which itself is run when
'the user selects the Insert Equation Reference command).
'
'Default example:
'{macrobutton MTPlaceRef {seq MTEqn \h}({seq MTChap \c \* Arabic}.{seq MTSec \c \* Arabic}
'.{seq MTEqn \c \* Arabic})}

'shared variables for communicating with MTInsertEqnNumDlg
Public gChapterNumber As String
Public gSectionNumber As String
Public gDlgCanceled As Boolean
Public gDontShowEqnNumWarning As Boolean

Option Explicit

Public Sub DlgMain()
Attribute DlgMain.VB_Description = "Inserts an equation number at the current insertion point."
    Dim Doc As Document
    Dim title As String
    
    'make sure locale DLL is available, exit if not
    If Not MTLib.CheckLocaleDLL() Then
        Exit Sub
    End If
    
    title = MTLib.GetUserString("!1301Insert Equation Number")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check to see if our cursor is in an OK location
    If MTLib.IsCursorPlacedOK(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If
    
    Set Doc = ActiveDocument
    'if there is a selection, collapse to start
    If Selection.Type <> wdSelectionIP Then
        Selection.Collapse wdCollapseStart
    End If
    
    If Val(Application.version) >= kWord2007 Then
        'If we are numbering an EB equation do it and exit
        If NumberExistingEBEqn Then GoTo abort
        'If the selection is in an EB equation display warning and exit
        If IsInEBEquation(title) Then GoTo abort
    End If
    
    InsertEqnNum

abort:
    'Enable screen redrawing
    MTLib.SetScreenUpdate True
End Sub

Public Sub InsertEqnNum()
    Dim fcState As Boolean
    Dim saveShowAll As Boolean
    Dim eqnFormat As String
    Dim Doc As Document
    Set Doc = ActiveDocument
    
    'if there is a selection, collapse to start
    If Selection.Type <> wdSelectionIP Then
        Selection.Collapse wdCollapseStart
    End If
    
    'check that an equation section has been inserted
    If Not CheckSectionNumber(Doc) Then
        GoTo abort
    End If
    
    'Disable screen redrawing
    MTLib.SetScreenUpdate False
    
    'If the selection ends on a paragraph marker, pull it back by 1
    'We want that to be within the current paragraph.
    If (Selection.start <> Selection.end) And _
       (Selection.end <> 0) And _
       (Selection.end <> Doc.Range.end) Then
       
        If Selection.Characters.Last = Strings.Chr(13) Then
            Selection.end = Selection.end - 1
        End If
    End If

    'record the current state of the field code view
    fcState = Doc.ActiveWindow.View.ShowFieldCodes
    
    'save ShowAll state
    saveShowAll = Doc.ActiveWindow.View.ShowAll
    Doc.ActiveWindow.View.ShowAll = False
    
    'insert a space just in case there is another field right before
    Selection.InsertAfter " "
    Selection.Collapse wdCollapseEnd
    
    'insert fieldchars, macrobutton (for references), and
    'field to increment eqn#
    Selection.Fields.Add Selection.Range, wdFieldMacroButton, "MTPlaceRef"
    Selection.moveLeft wdCharacter, 1, wdExtend
    If fcState = False Then
        Selection.Fields.ToggleShowCodes
    End If
    Selection.Collapse wdCollapseEnd
    Selection.moveLeft wdCharacter, 1
    Selection.Fields.Add Selection.Range, wdFieldSequence, "MTEqn " + Strings.ChrW(&H5C) + "h"

    'insert equation number
    eqnFormat = MTEqnNumFormat.GetEqnNumFormat(Doc)
    MTEqnNumFormat.InsertEquationNumber Doc, eqnFormat
    
    'reset the ShowAll state
    Doc.ActiveWindow.View.ShowAll = saveShowAll
    
    'reset the field code view to where the user had it
    Doc.ActiveWindow.View.ShowFieldCodes = fcState

    'update field by selecting it & calling Update
    Selection.MoveRight wdCharacter, 1, wdExtend
    
    'update new field, and then existing equation numbers (don't force)
    Selection.Fields.update
    MTUpdateFields.UpdateFields mt_RANGE_DOCUMENT, False
        
    'delete the extra space we created
    Selection.Collapse wdCollapseStart
    Selection.moveLeft wdCharacter, 1, wdExtend
    Selection.delete
    'move the cursor back to the right place
    Selection.MoveRight wdCharacter, 1, wdExtend
    Selection.Collapse wdCollapseEnd
abort:
    'Enable screen redrawing
    MTLib.SetScreenUpdate True
End Sub

'Checks if section number has been inserted in this document.
'If not, asks user if they want one inserted at start of document.
'Returns True if OK, False to Cancel insertion.
Public Function CheckSectionNumber(Doc As Document) As Boolean
    Dim eqnFormat As String
    Dim curRange As Range
    Dim resetRange As Boolean, doInsert As Boolean
    Dim break As BreakDlgInfo

    CheckSectionNumber = True

    If MTLib.DocPropertyExists(Doc, mtprop_EQUATION_SECTION_CHECKED) Then
        Exit Function
    End If

    'see if we need to insert an initial chapter/section break
    If CheckSEQValue("MTSec " + Strings.ChrW(&H5C) + "c") = "0" Then
        gChapterNumber = "1"
        gSectionNumber = "1"
        doInsert = False

        'if user doesn't want to see warning, insert explicit break silently
        If GetDontShowEqnNumWarning() Then
            doInsert = True
        Else
            'if the current format doesn't show chapter & section number, insert silently
            eqnFormat = MTEqnNumFormat.GetEqnNumFormat(Doc)
            If InStr(1, eqnFormat, "#C", vbBinaryCompare) = 0 And _
                InStr(1, eqnFormat, "#S", vbBinaryCompare) = 0 Then
                doInsert = True
            Else
                MTInsertEqnNumDlg.Show
                If gDlgCanceled Then
                    CheckSectionNumber = False
                Else
                    doInsert = True
                    If gDontShowEqnNumWarning Then
                        SetDontShowEqnNumWarning True
                    End If
                End If
            End If
        End If

        'insert the section heading at start of doc
        If doInsert Then
            MTLib.SetScreenUpdate False
            
            resetRange = False
            If Selection.Range.start > 0 Then
                Set curRange = Selection.Range
                resetRange = True
            End If
            Selection.StartOf wdStory, wdMove

            'insert explicit chapter & section break
            break.hasChapter = True
            break.isExplicitChapterNumber = True
            break.chapterNumber = gChapterNumber
            break.isExplicitSectionNumber = True
            break.sectionNumber = gSectionNumber
            MTSecNum.InsertNewEquationSection Doc, break
            If resetRange Then curRange.Select
            
            MTLib.SetScreenUpdate True
        End If
    End If

    'reset globals
    gChapterNumber = ""
    gSectionNumber = ""
End Function

'Sets eqn num warning value in registry
Public Sub SetDontShowEqnNumWarning(state As Boolean)
    Dim stateStr As String
    If state Then
        stateStr = "1"
    Else
        stateStr = "0"
    End If
    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNNUM_WARNING, stateStr
End Sub

'Returns True if user doesn't want to see warning
Public Function GetDontShowEqnNumWarning() As Boolean
    Dim stateStr As String
    stateStr = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNNUM_WARNING)
    GetDontShowEqnNumWarning = (stateStr = "1")
End Function

'Sets eqn num warning value in registry
Public Sub SetDontShowEqnRefWarning(state As Boolean)
    Dim stateStr As String
    If state Then
        stateStr = "1"
    Else
        stateStr = "0"
    End If
    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNREFDLG, stateStr
End Sub

'Returns True if user doesn't want to see warning
Public Function GetDontShowEqnRefWarning() As Boolean
    Dim stateStr As String
    stateStr = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_EQNREFDLG)
    GetDontShowEqnRefWarning = (stateStr = "1")
End Function

'Returns value of SEQ value at current insertion point
Private Function CheckSEQValue(name As String) As String
    Dim sectionValue As String
    Dim fcState As Boolean

    CheckSEQValue = ""
    
    On Error GoTo abort
    MTLib.SetScreenUpdate False
    
    Selection.Collapse direction:=wdCollapseEnd

    'get field code state and turn field codes off
    fcState = ActiveWindow.View.ShowFieldCodes
    If fcState Then
        ActiveWindow.View.ShowFieldCodes = False
    End If

    With Selection
    'insert a field using our section field identifier, & get its value
        .InsertAfter (" ") 'extra space so word selection works
        .Collapse wdCollapseEnd

        .Fields.Add Range:=.Range, Type:=wdFieldSequence, _
            Text:=name$
        .moveLeft wdWord, 1, wdExtend
        CheckSEQValue = .Range.Text
        'select extra space
        .moveLeft wdCharacter, 1, wdExtend
        .delete
    End With
    
abort:
    'reset the field code view to where the user had it
    ActiveWindow.View.ShowFieldCodes = fcState

    MTLib.SetScreenUpdate True
End Function


