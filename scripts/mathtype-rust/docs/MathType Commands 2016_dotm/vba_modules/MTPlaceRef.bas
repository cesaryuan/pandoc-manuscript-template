Attribute VB_Name = "MTPlaceRef"
'MTPlaceRef 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTPlaceRef.bas 43    5/06/14 9:54a Jimm $
'=====================================================================
'This macro is run when an equation number is double-clicked on.
'It checks to make sure the bookmark "MTReference", which is defined
'by the "MTMarkRef" macro, exists, then checks to see if the
'currently selected equation number has a bookmark associated with
'it, if not it creates one. Then it places a reference to the
'equation's bookmark at the location marked by "MTReference". The
'reference displays the text of the equation number, can be updated
'if that text changes, and contains a gotobutton that goes to the
'original equation when double-clicked on.

'Default example:  {gotobutton ZEqnNum123456 {ref  ZEqnNum123456 \!}}

Option Explicit

Public Sub DlgMain()
Attribute DlgMain.VB_Description = "This macro is used in conjunction with MTMarkRef."
Attribute DlgMain.VB_ProcData.VB_Invoke_Func = "TemplateProject.MTPlaceRef.MAIN"
    Dim lSeq$, eqnBkMrk$
    Dim numLength As Long, move As Long
    Dim myResult As Long, paneID As Long
    Dim fcState As Boolean
    Dim paneNum$
    Dim aField As Field
    Dim title As String

    On Error Resume Next

    'save typing-replaces-text setting
    Dim saveTypingReplaces As Boolean
    saveTypingReplaces = options.ReplaceSelection
    options.ReplaceSelection = True

    title = MTLib.GetUserString("!1901Insert Equation Reference")
    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo done
    End If

    'get the localized spelling of the Seq field name
    lSeq$ = MTLib.GetLocaleStr("!0102SEQ")

    MTLib.SetScreenUpdate False

    'save the current view
    Dim saveView As WdViewType
    saveView = ActiveWindow.View.Type

    'update selected equation so we're referring to the current number
    ActiveDocument.Fields.update

    'record the current field code state so we can restore it later
    fcState = ActiveWindow.View.ShowFieldCodes

    'if "MTReference" bookmark has been placed...
    If ActiveDocument.Bookmarks.Exists("MTReference") Then
        With Selection
            .MoveRight Unit:=wdCharacter, count:=1
            .moveLeft Unit:=wdCharacter, count:=1, Extend:=wdExtend
            .Fields.ToggleShowCodes    'turn field display on for eqn

            'deselect equation
            .MoveRight Unit:=wdCharacter, count:=1, Extend:=wdMove
            'move inside field characters
            .moveLeft Unit:=wdCharacter, count:=1, Extend:=wdMove

            'select entire equation number
            While InStr(Strings.UCase$(.Text), Strings.UCase$(lSeq$) + " MTEQN " + Strings.ChrW(&H5C) + "H") = 0
                move = .moveLeft(wdCharacter, 1, wdExtend)
                If move = 0 Then GoTo err   'abort if we can't move selection
                numLength = numLength + 1
            Wend

            'deselect equation number
            .MoveRight Unit:=wdCharacter, count:=1, Extend:=wdMove

            'reselect all but the {seq MTEqn \h} field
            .moveLeft Unit:=wdCharacter, count:=numLength - 1, Extend:=wdExtend
        End With

        'Default example:  for the equation number
        '"{macrobutton MTPlaceRef {seq MTEqn \h}({seq MTSec \c}.
        '{seq MTEqn \c})}" the bookmark would be assigned to only the
        '"({seq MTSec \c}.{seq MTEqn \c})" part
        'note: all sequence fields referenced with reference fields
        'MUST be current sequence fields (which include a \c), since
        'referencing a regular sequence field causes the sequence
        'counter to increment. To accomplish this, the fields that
        'actually cause the sequence to increment are hidden (with
        '\h). This is a workaround for a problem with the way
        'Microsoft Word handles sequence referencing.

        eqnBkMrk$ = GetBookmarkName()
        ActiveDocument.Bookmarks.Add name:=eqnBkMrk$ 'insert new bookmark
        'move outside the outer field
        Selection.MoveRight Unit:=wdCharacter, count:=2, Extend:=wdMove
        'select field
        Selection.moveLeft Unit:=wdCharacter, count:=2, Extend:=wdExtend

        'turn field codes off for all selected fields
        '(we had to do it this way in case we accidently selected an extra
        'field, like an equation)
        For Each aField In Selection.Fields
            aField.ShowCodes = False
        Next

        'go to "MTReference" bookmark, use original pane if property exists
        'allows user to insert refs in one pane & dbl-click in the other
        paneNum$ = MTLib.ReadDocPropString$(ActiveDocument, mtprop_EQNREFPANE)
        If paneNum$ <> "" Then
            paneID = Val(paneNum$)
            If paneID > 0 And paneID <= ActiveWindow.Panes.count Then
                ActiveWindow.Panes(paneID).Activate
            End If
            MTLib.DeleteDocProperty ActiveDocument, mtprop_EQNREFPANE
        End If
        ' the MTReference bookmark contains a "equation reference goes here",
        ' which is selected by this next command
        ActiveDocument.Bookmarks("MTReference").Select

        With Selection

            'the MTReference bookmark was formatted with italics, so remove the formatting
            .Range.Italic = False

            'insert GoToButton to jump to the equation...
            'note that this will delete the MTReference bookmark
            .Fields.Add Range:=Selection.Range, Type:=wdFieldGoToButton, Text:=eqnBkMrk$ + " "
            .moveLeft Unit:=wdCharacter, count:=1, Extend:=wdExtend
            If ActiveWindow.View.ShowFieldCodes = False Then
                .Fields.ToggleShowCodes
            End If
            .Collapse direction:=wdCollapseEnd
            .moveLeft Unit:=wdCharacter, count:=1, Extend:=wdMove

            'insert nested reference field to display equation text
            '(\! causes seq not to be reevaluated at current location)
            .Fields.Add Range:=.Range, Type:=wdFieldRef, _
                Text:=eqnBkMrk$ + " " + Strings.ChrW(&H5C) + "* Charformat " + Strings.ChrW(&H5C) + "!"

            'select the field
            .MoveRight Unit:=wdCharacter, count:=1, Extend:=wdExtend

            'If field codes are displayed turn them off
            .Fields.ToggleShowCodes

            'unselect field
            .MoveRight Unit:=wdCharacter, count:=1, Extend:=wdMove

        End With

        If fcState <> ActiveWindow.View.ShowFieldCodes Then
            ActiveWindow.View.ShowFieldCodes = fcState
        End If
    Else
        GoTo err
    End If
    GoTo done

err:
    '"MTReference" does not exist or we can't find our SEQ field
    Beep
    MsgBox MTLib.GetUserString("!2000No location was marked for an equation reference. First choose the Insert Equation Reference command on the MathType menu, then double-click on an equation number."), _
        vbOKOnly + vbCritical, title

done:
    'restore view
    ActiveWindow.View.Type = saveView
    options.ReplaceSelection = saveTypingReplaces

    MTLib.SetScreenUpdate True
End Sub

'Returns bookmark name, either existing one that matches selection
'or a new one.
Function GetBookmarkName() As String
    Dim abookmark As Bookmark
    Dim found As Boolean

    found = False
    For Each abookmark In ActiveDocument.Bookmarks
        If abookmark.Range = Selection.Range Then
            found = True
            Exit For
        End If
    Next

    If found Then
        GetBookmarkName = abookmark.name
    Else
        GetBookmarkName = CreateBookmarkName()
    End If
End Function
'Create bookmark with name ZEQNNUM + 6 digit extension.
Function CreateBookmarkName() As String
    Dim randNum As Long
    Dim randStr As String
    Dim bookmarkName As String

    Randomize
    randNum = Int((999999 - 100000 + 1) * Rnd + 100000)
    randStr = Strings.Trim$(Strings.right$(Conversion.str(randNum), 6))
    bookmarkName = "ZEqnNum" + randStr
    'verify that bookmark does not exist
    While ActiveDocument.Bookmarks.Exists(bookmarkName)
        'increment number by 1 & try again
        randStr = Strings.Trim$(Strings.right$(Conversion.str(Val(randStr) + 1), 6))
        bookmarkName = "ZEqnNum" + randStr
    Wend
    CreateBookmarkName = bookmarkName
End Function
