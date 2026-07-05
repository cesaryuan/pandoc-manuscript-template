Attribute VB_Name = "MTUpdateFields"
'MTUpdateFields: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTUpdateFields.bas 31    10/11/11 2:12p Jimm $
'=====================================================================

'This macro runs Word's UpdateFields command, which updates
'all equation numbers (& any other fields). It lets the user
'decide whether to update the current selection (if there is
'one) or the whole document. It also runs UpdateFields twice,
'as forward references to equation numbers aren't updated
'correctly until UpdateFields is run a second time.
Option Explicit

'Updates all fields in active document/selection (if there is one).
Sub DlgMain()
    Dim sel As Long
    Dim title As String
    
    title = MTLib.GetUserString("!2301Update Equation Numbers")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If

    'default to entire document
    sel = mt_RANGE_DOCUMENT
    
    'if text selected, update selection
    If Selection.Type = wdSelectionNormal Then
        sel = mt_RANGE_SELECTION
    End If
    
    'pass True to force update
    UpdateFields sel, True
abort:
End Sub
'Updates fields in document, saving & restoring current selection.
'sel = mt_RANGE_SELECTION or mt_RANGE_DOCUMENT
'force = True to force update, else check for deferred updates
Public Sub UpdateFields(sel As Long, force As Boolean)
    ActiveDocument.Bookmarks.Add name:="MTUpdateHome"

    UpdateFieldCodes sel, force

    ActiveDocument.Bookmarks("MTUpdateHome").Select
    ActiveDocument.Bookmarks("MTUpdateHome").delete
End Sub

'Updates fields in active document.
'Checks our Property to see if should be skipped, unless "force"
'parameter is True, in which case it always updates.
'Updates twice to make sure forward references are correct.
'If not forced, times how long the update takes, if >2 secs ask user if
'they want to defer the updating. Only ask if 'Dont Show' hasn't been
'previously checked, & don't ask if already deferring updates.
Private Sub UpdateFieldCodes(sel As Long, force As Boolean)
    Dim update As Boolean
    Dim defer As String
    Dim dontShow As String
    Dim start As Long
    Dim oldAlertState As Long
    Dim aStory As Range

    update = force
    If update = False Then
        defer = MTLib.ReadDocPropString$(ActiveDocument, mtprop_DEFER_FIELD_UPDATE)
        If (defer <> "1") Then update = True
    End If
    
    If update Then
      start = MTGetTickCount()
      MTLib.SetScreenUpdate False
      oldAlertState = Application.DisplayAlerts
      Application.DisplayAlerts = wdAlertsNone
        
      If sel = mt_RANGE_SELECTION Then
         Selection.Fields.update
         Selection.Fields.update
      Else
         For Each aStory In ActiveDocument.StoryRanges
            Do
                If (0 = aStory.Fields.update) Then
                    aStory.Fields.update
                End If
                'if this story has a valid NextRange, process it
                Set aStory = aStory.NextStoryRange
            Loop While (IsObjectValid(aStory))
        Next

      End If
        MTLib.SetScreenUpdate True

        If (force = False) And ((MTGetTickCount() - start) > 2000) Then
            dontShow = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_SLOWEQNUPDATE)
            defer = MTLib.ReadDocPropString(ActiveDocument, mtprop_DEFER_FIELD_UPDATE)
            If (dontShow <> "1") And (defer <> "1") Then
                MTUpdateEqnNums.Show
            End If
        End If
    End If
End Sub

