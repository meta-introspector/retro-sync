<?xml version="1.0" encoding="UTF-8"?>
<!--
  sacem.xsl — Transforms Retrosync canonical CWR-XML into SACEM
  eDépôt XML format (sacem.fr/depot/1.0).
  Society: SACEM (FR).  CWR code: 058.
-->
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:cwr="https://retrosync.media/xml/cwr/1"
  xmlns:sacem="https://www.sacem.fr/depot/1.0">

  <xsl:output method="xml" encoding="UTF-8" indent="yes"/>
  <xsl:param name="declarant_ipi" select="''"/>
  <xsl:param name="depot_date"    select="'19700101'"/>

  <xsl:template match="/cwr:WorkRegistrations">
    <sacem:Depot version="1.0">
      <sacem:Entete>
        <sacem:Declarant><xsl:value-of select="$declarant_ipi"/></sacem:Declarant>
        <sacem:DateDepot><xsl:value-of select="$depot_date"/></sacem:DateDepot>
        <sacem:NombreOeuvres><xsl:value-of select="count(cwr:Work)"/></sacem:NombreOeuvres>
      </sacem:Entete>
      <sacem:Oeuvres>
        <xsl:apply-templates select="cwr:Work"/>
      </sacem:Oeuvres>
    </sacem:Depot>
  </xsl:template>

  <xsl:template match="cwr:Work">
    <sacem:Oeuvre>
      <sacem:ISWC><xsl:value-of select="cwr:Iswc"/></sacem:ISWC>
      <sacem:Titre><xsl:value-of select="cwr:Title"/></sacem:Titre>
      <sacem:Langue><xsl:value-of select="cwr:Language"/></sacem:Langue>
      <sacem:TypeArrangement><xsl:value-of select="cwr:MusicArrangement"/></sacem:TypeArrangement>
      <sacem:DroitsGrands><xsl:value-of select="cwr:GrandRightsInd"/></sacem:DroitsGrands>
      <sacem:Auteurs>
        <xsl:apply-templates select="cwr:Writers/cwr:Writer"/>
      </sacem:Auteurs>
      <sacem:Editeurs>
        <xsl:apply-templates select="cwr:Publishers/cwr:Publisher"/>
      </sacem:Editeurs>
      <xsl:if test="cwr:AlternateTitles/cwr:AlternateTitle">
        <sacem:TitresAlternatifs>
          <xsl:apply-templates select="cwr:AlternateTitles/cwr:AlternateTitle"/>
        </sacem:TitresAlternatifs>
      </xsl:if>
      <xsl:if test="cwr:PerformingArtists/cwr:PerformingArtist">
        <sacem:Interpretes>
          <xsl:apply-templates select="cwr:PerformingArtists/cwr:PerformingArtist"/>
        </sacem:Interpretes>
      </xsl:if>
    </sacem:Oeuvre>
  </xsl:template>

  <xsl:template match="cwr:Writer">
    <sacem:Auteur>
      <sacem:Nom><xsl:value-of select="cwr:LastName"/></sacem:Nom>
      <sacem:Prenom><xsl:value-of select="cwr:FirstName"/></sacem:Prenom>
      <sacem:IPI><xsl:value-of select="cwr:IpiCae"/></sacem:IPI>
      <sacem:Fonction><xsl:value-of select="cwr:Role"/></sacem:Fonction>
      <sacem:Quote><xsl:value-of select="cwr:SharePct"/></sacem:Quote>
      <sacem:Societe><xsl:value-of select="cwr:Society"/></sacem:Societe>
    </sacem:Auteur>
  </xsl:template>

  <xsl:template match="cwr:Publisher">
    <sacem:Editeur>
      <sacem:Nom><xsl:value-of select="cwr:Name"/></sacem:Nom>
      <sacem:IPI><xsl:value-of select="cwr:IpiCae"/></sacem:IPI>
      <sacem:Quote><xsl:value-of select="cwr:SharePct"/></sacem:Quote>
    </sacem:Editeur>
  </xsl:template>

  <xsl:template match="cwr:AlternateTitle">
    <sacem:TitreAlternatif type="{cwr:TitleType}">
      <xsl:value-of select="cwr:Title"/>
    </sacem:TitreAlternatif>
  </xsl:template>

  <xsl:template match="cwr:PerformingArtist">
    <sacem:Interprete>
      <sacem:Nom><xsl:value-of select="cwr:LastName"/></sacem:Nom>
      <xsl:if test="cwr:Isni != ''">
        <sacem:ISNI><xsl:value-of select="cwr:Isni"/></sacem:ISNI>
      </xsl:if>
    </sacem:Interprete>
  </xsl:template>
</xsl:stylesheet>
