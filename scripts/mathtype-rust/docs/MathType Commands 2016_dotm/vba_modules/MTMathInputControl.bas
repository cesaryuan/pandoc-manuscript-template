Attribute VB_Name = "MTMathInputControl"
'MTMathInputControl
'=====================================================================
' (c) Copyright 2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTMathInputControl.bas 13    10/11/11 2:12p Jimm $
'=====================================================================
Option Explicit

Public Sub DlgMain()

#If Word Then

    Dim stat As Long
    stat = MTInsertHandwrittenMath()
    If stat <> mtOK Then
        MsgBox MTLib.GetUserString("!1685Error opening Math Input Panel"), vbCritical, MTLib.GetUserString("!1684Open Math Input Panel")
        Else
                MTIncrementStatisticBy "MICWord", 1
    End If
        
#End If

End Sub
