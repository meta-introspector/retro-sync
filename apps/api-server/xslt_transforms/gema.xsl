<?xml version="1.0" encoding="UTF-8"?>
<!--
  gema.xsl — Transforms Retrosync canonical CWR-XML into GEMA
  WorkRegistration portal XML format (gema.de/meldung/1.0).
  Society: GEMA (DE).  CWR code: 035.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:gema="https://www.gema.de/meldung/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="sender_ipi" select="''"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <gema:Meldung version="1.0">
      <gema:Absender>
        <gema:IPI><xsl:value-of select="$sender_ipi"/></gema:IPI>
      </gema:Absender>
      <gema:Werke>
        <xsl:apply-templates select="cwr:Work"/>
      </gema:Werke>
    </gema:Meldung>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <gema:Werk>
      <gema:ISWC><xsl:value-of select="cwr:Iswc"/></gema:ISWC>
      <gema:Werktitel><xsl:value-of select="cwr:Title"/></gema:Werktitel>
      <gema:Sprache><xsl:value-of select="cwr:Language"/></gema:Sprache>
      <gema:Arrangement><xsl:value-of select="cwr:MusicArrangement"/></gema:Arrangement>
      <xsl:if test="cwr:OpusNumber != ''">
        <gema:Opusnummer><xsl:value-of select="cwr:OpusNumber"/></gema:Opusnummer>
      </xsl:if>
      <xsl:if test="cwr:CatalogueNumber != ''">
        <gema:Katalognummer><xsl:value-of select="cwr:CatalogueNumber"/></gema:Katalognummer>
      </xsl:if>
      <gema:GrandRights><xsl:value-of select="cwr:GrandRightsInd"/></gema:GrandRights>
      <gema:Urheber>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </gema:Urheber>
      <gema:Verleger>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </gema:Verleger>
      <xsl:if test="cwr:AlternateTitles/cwr:AlternateTitle">
        <gema:AlternativTitel>
          <xsl:apply-templates select="cwr:AlternateTitles/cwr:AlternateTitle"/>
        </gema:AlternativTitel>
      </xsl:if>
      <xsl:if test="cwr:PerformingArtists/cwr:PerformingArtist">
        <gema:Interpreten>
          <xsl:apply-templates select="cwr:PerformingArtists/cwr:PerformingArtist"/>
        </gema:Interpreten>
      </xsl:if>
    </gema:Werk>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <gema:Urheber>
      <gema:Nachname><xsl:value-of select="cwr:LastName"/></gema:Nachname>
      <gema:Vorname><xsl:value-of select="cwr:FirstName"/></gema:Vorname>
      <gema:IPI><xsl:value-of select="cwr:IpiCae"/></gema:IPI>
      <gema:Rolle><xsl:value-of select="cwr:Role"/></gema:Rolle>
      <gema:Anteil><xsl:value-of select="cwr:SharePct"/></gema:Anteil>
      <gema:Gesellschaft><xsl:value-of select="cwr:Society"/></gema:Gesellschaft>
    </gema:Urheber>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <gema:Verleger>
      <gema:Name><xsl:value-of select="cwr:Name"/></gema:Name>
      <gema:IPI><xsl:value-of select="cwr:IpiCae"/></gema:IPI>
      <gema:Anteil><xsl:value-of select="cwr:SharePct"/></gema:Anteil>
    </gema:Verleger>
  </xsl:template>

  <xsl:template match="cwr:AlternateTitle">
    <gema:Titel typ="{cwr:TitleType}">
      <xsl:value-of select="cwr:Title"/>
    </gema:Titel>
  </xsl:template>

  <xsl:template match="cwr:PerformingArtist">
    <gema:Interpret>
      <gema:Name><xsl:value-of select="cwr:LastName"/></gema:Name>
      <xsl:if test="cwr:Isni != ''">
        <gema:ISNI><xsl:value-of select="cwr:Isni"/></gema:ISNI>
      </xsl:if>
    </gema:Interpret>
  </xsl:template>
</xsl:stylesheet>
